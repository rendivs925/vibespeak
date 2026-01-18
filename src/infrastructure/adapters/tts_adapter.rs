use crate::domain::services::TextToSpeechService;
use crate::shared::{Error, Result};
use async_trait::async_trait;
use std::process::Command;
use uuid::Uuid;

/// Voice configuration for TTS
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    pub name: String,
    pub pitch: f32,      // 0.5 - 2.0
    pub rate: f32,       // 0.5 - 2.0 (words per minute multiplier)
    pub volume: f32,     // 0.0 - 1.0
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            pitch: 1.0,
            rate: 1.0,
            volume: 0.8,
        }
    }
}

pub struct TtsAdapter {
    sample_rate: u32,
    default_voice: VoiceConfig,
    use_piper: bool,
}

impl TtsAdapter {
    pub fn new() -> Result<Self> {
        // Check for Piper TTS engine (required)
        let system_piper = Command::new("which")
            .arg("piper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let local_piper_exists = std::path::Path::new("./piper/piper").exists()
            || std::path::Path::new("piper/piper").exists();

        let use_piper = system_piper || local_piper_exists;

        tracing::info!("TTS detection: system piper={}, local piper exists={}, use_piper={}",
            system_piper, local_piper_exists, use_piper);

        if use_piper {
            tracing::info!("TTS: Piper neural TTS detected - providing high-quality natural voices with Amy model");
        } else {
            tracing::warn!("TTS: Piper not found. Please install Piper TTS for voice synthesis.");
            tracing::info!("Install Piper from: https://github.com/rhasspy/piper");
        }

        Ok(Self {
            sample_rate: 44100,
            default_voice: VoiceConfig::default(),
            use_piper,
        })
    }

    /// Generate speech using Piper TTS with optimized long text handling
    async fn synthesize_piper(&self, text: &str, _voice: &VoiceConfig) -> Result<Vec<i16>> {
        // Preprocess text for better synthesis of long paragraphs
        let processed_text = self.preprocess_text_for_tts(text);
        tracing::debug!("Processing text for TTS: {} chars -> {} chars", text.len(), processed_text.len());

        // Check if piper is available
        let piper_available = Command::new("which")
            .arg("piper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("./piper/piper")
                .arg("--help")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        if !piper_available {
            return Err(Error::Infrastructure("Piper TTS not found. Please install Piper TTS.".to_string()));
        }

        // Create temporary file for WAV output
        let temp_path = format!("/tmp/vibespeak_tts_{}.wav", Uuid::new_v4());

        // Use only the en_US-amy-medium voice model for all voices
        // This is the single, high-quality English female voice model from Piper samples
        let voice_model = "en_US-amy-medium.onnx";

        // Check if the voice model exists in common locations
        let model_paths = vec![
            format!("./models/{}", voice_model),
            format!("/usr/local/share/piper/{}", voice_model),
            format!("/usr/share/piper/{}", voice_model),
            voice_model.to_string(), // Try as-is (assume it's in PATH or absolute path)
        ];

        let mut model_found = false;
        let mut actual_model_path = voice_model.to_string();

        for path in model_paths {
            if std::path::Path::new(&path).exists() {
                actual_model_path = path;
                model_found = true;
                break;
            }
        }

        if !model_found {
            tracing::warn!("Piper voice model '{}' not found in common locations. Make sure to download Piper voice models.", voice_model);
            tracing::info!("Download voice models from: https://huggingface.co/rhasspy/piper-voices/tree/v1.0.0");
        }

        // Choose piper command (system or local with absolute path)
        let piper_cmd = if Command::new("which")
            .arg("piper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "piper".to_string()
        } else {
            // Use absolute path to piper binary
            let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let piper_path = current_dir.join("piper").join("piper");
            piper_path.to_string_lossy().to_string()
        };

        // Use Piper with default settings for en_US-amy-medium model
        // This matches the direct command: piper --model en_US-amy-medium.onnx --output_file file.wav
        let mut child = Command::new(piper_cmd)
            .args([
                "--espeak_data", "./piper/espeak-ng-data",
                "--model", &actual_model_path,
                "--output_file", &temp_path,
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::Infrastructure(format!("Failed to start Piper: {}", e)))?;

        // Write processed text to stdin
        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            stdin.write_all(processed_text.as_bytes())
                .map_err(|e| Error::Infrastructure(format!("Failed to write text to Piper: {}", e)))?;
        }

        // Wait for completion
        let result = child.wait_with_output()
            .map_err(|e| Error::Infrastructure(format!("Failed to wait for Piper: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(Error::Infrastructure(format!("Piper failed: {}", stderr)));
        }

        // Read the WAV file and extract high-quality PCM data
        let samples = self.read_wav_file(&temp_path)?;
        tracing::debug!("Generated {} PCM samples for high-quality female voice synthesis", samples.len());

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(samples)
    }



    /// Preprocess text for better TTS synthesis of long paragraphs
    fn preprocess_text_for_tts(&self, text: &str) -> String {
        let mut processed = text.to_string();

        // Clean up excessive whitespace
        processed = processed.split_whitespace().collect::<Vec<&str>>().join(" ");

        // Add small pauses after sentences for better listening experience
        // Piper handles sentence boundaries automatically, but we can ensure clean text
        processed = processed
            .replace("  ", " ")  // Remove double spaces
            .replace(" ,", ",")  // Fix spacing around commas
            .replace(" .", ".")  // Fix spacing around periods
            .replace(" !", "!")  // Fix spacing around exclamation marks
            .replace(" ?", "?")  // Fix spacing around question marks
            .trim()
            .to_string();

        // Piper can handle very long text well, but for optimal performance and user experience,
        // we'll limit to reasonable lengths while preserving sentence boundaries
        if processed.len() > 15000 {
            processed = processed.chars().take(15000).collect();
            // Try to end at a sentence boundary for better listening experience
            if let Some(last_sentence_end) = processed.rfind(|c: char| c == '.' || c == '!' || c == '?') {
                if last_sentence_end > processed.len() / 2 {
                    processed = processed.chars().take(last_sentence_end + 1).collect();
                }
            }
            tracing::info!("Long text truncated to {} characters for optimal TTS performance", processed.len());
        }

        processed
    }

    /// Read WAV file and return PCM samples
    fn read_wav_file(&self, path: &str) -> Result<Vec<i16>> {
        use std::fs::File;
        use std::io::{BufReader, Read, Seek, SeekFrom};

        let file = File::open(path)
            .map_err(|e| Error::Infrastructure(format!("Failed to open WAV file: {}", e)))?;
        let mut reader = BufReader::new(file);

        // Read WAV header (simplified parser for standard WAV)
        let mut header = [0u8; 44];
        reader.read_exact(&mut header)
            .map_err(|e| Error::Infrastructure(format!("Failed to read WAV header: {}", e)))?;

        // Verify RIFF header
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(Error::Infrastructure("Invalid WAV file format".to_string()));
        }

        // Find data chunk (may not be at offset 44 for all WAV files)
        let mut data_start = 12;
        loop {
            reader.seek(SeekFrom::Start(data_start as u64))
                .map_err(|e| Error::Infrastructure(format!("Failed to seek in WAV: {}", e)))?;

            let mut chunk_header = [0u8; 8];
            if reader.read_exact(&mut chunk_header).is_err() {
                break;
            }

            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_le_bytes([chunk_header[4], chunk_header[5], chunk_header[6], chunk_header[7]]);

            if chunk_id == b"data" {
                // Read PCM data
                let mut data = vec![0u8; chunk_size as usize];
                reader.read_exact(&mut data)
                    .map_err(|e| Error::Infrastructure(format!("Failed to read PCM data: {}", e)))?;

                // Convert bytes to i16 samples (assuming 16-bit little-endian)
                let samples: Vec<i16> = data
                    .chunks_exact(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();

                return Ok(samples);
            }

            data_start += 8 + chunk_size as usize;
        }

        Err(Error::Infrastructure("No data chunk found in WAV file".to_string()))
    }


}

#[async_trait]
impl TextToSpeechService for TtsAdapter {
    async fn synthesize(&self, text: &str, _voice: Option<&str>) -> Result<Vec<i16>> {
        tracing::info!("Synthesizing text: '{}' using en_US-amy-medium model", text);

        // Always use the same voice config for en_US-amy-medium model
        // Voice parameter is ignored - only Amy model is used
        let voice_config = VoiceConfig {
            name: "amy".to_string(),
            pitch: 1.0,
            rate: 1.0,
            volume: 0.8,
        };

        // Use Piper TTS (required)
        if self.use_piper {
            match self.synthesize_piper(text, &voice_config).await {
                Ok(samples) => return Ok(samples),
                Err(e) => {
                    tracing::error!("Piper TTS synthesis failed: {}", e);
                    return Err(e);
                }
            }
        }

        // Piper not available - return error
        Err(Error::Infrastructure("Piper TTS not available. Please install Piper TTS.".to_string()))
    }

    async fn get_available_voices(&self) -> Result<Vec<String>> {
        // Only Amy voice model is available
        let voices = vec![
            "default".to_string(),
            "amy".to_string(),
        ];

        Ok(voices)
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!(
            "TTS adapter initialized - piper: {}, sample_rate: {}",
            self.use_piper,
            self.sample_rate
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("TTS adapter shutdown");
        Ok(())
    }
}
