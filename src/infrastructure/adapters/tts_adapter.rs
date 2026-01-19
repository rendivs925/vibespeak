use crate::domain::services::TextToSpeechService;
use crate::shared::{Error, Result};
use async_trait::async_trait;
use std::process::Command;
use uuid::Uuid;

// Voice configuration removed - only Piper Amy model is supported

pub struct TtsAdapter {
    sample_rate: u32,
}

impl TtsAdapter {
    pub fn new() -> Result<Self> {
        // Verify Piper TTS engine is available (required - no fallback)
        let piper_available = Self::check_piper_availability();

        if !piper_available {
            return Err(Error::Infrastructure(
                "Piper TTS not found. Piper TTS is required for voice synthesis.\n\
                 Please ensure Piper is installed and available in PATH or in ./piper/piper"
                    .to_string(),
            ));
        }

        tracing::info!("TTS: Piper neural TTS with Amy voice model ready for high-quality natural voice synthesis");

        Ok(Self { sample_rate: 44100 })
    }

    /// Check if Piper TTS is available
    fn check_piper_availability() -> bool {
        // Check system piper
        if Command::new("which")
            .arg("piper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Check local piper
        if std::path::Path::new("./piper/piper").exists()
            || std::path::Path::new("piper/piper").exists()
        {
            return true;
        }

        false
    }

    /// Generate speech using Piper TTS (only TTS engine supported)
    async fn synthesize_piper(&self, text: &str) -> Result<Vec<i16>> {
        // Preprocess text for better synthesis
        let processed_text = self.preprocess_text_for_tts(text);

        // Create temporary file for WAV output
        let temp_path = format!("/tmp/vibespeak_tts_{}.wav", Uuid::new_v4());

        // Always use en_US-amy-medium model (only voice model supported)
        let voice_model_path = "./models/en_US-amy-medium.onnx".to_string();

        // Verify model exists
        if !std::path::Path::new(&voice_model_path).exists() {
            return Err(Error::Infrastructure(
                format!("Piper voice model not found at: {}. Please ensure the Amy voice model is installed.", voice_model_path)
            ));
        }

        // Get piper command path
        let piper_cmd = if Command::new("which")
            .arg("piper")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "piper".to_string()
        } else {
            // Use local piper binary
            let current_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let piper_path = current_dir.join("piper").join("piper");
            piper_path.to_string_lossy().to_string()
        };

        // Set LD_LIBRARY_PATH to include piper libraries
        let mut command = Command::new(&piper_cmd);
        command
            .args([
                "--espeak_data",
                "./piper/espeak-ng-data",
                "--model",
                &voice_model_path,
                "--output_file",
                &temp_path,
            ])
            .env("LD_LIBRARY_PATH", "./piper/lib")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| Error::Infrastructure(format!("Failed to start Piper: {}", e)))?;

        // Write text to Piper's stdin
        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            stdin.write_all(processed_text.as_bytes()).map_err(|e| {
                Error::Infrastructure(format!("Failed to write text to Piper: {}", e))
            })?;
        }

        // Wait for Piper to complete
        let result = child
            .wait_with_output()
            .map_err(|e| Error::Infrastructure(format!("Failed to wait for Piper: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(Error::Infrastructure(format!(
                "Piper TTS failed: {}",
                stderr
            )));
        }

        // Read WAV file and extract PCM samples
        let samples = self.read_wav_file(&temp_path)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        tracing::debug!(
            "Generated {} PCM samples using Piper Amy voice model",
            samples.len()
        );
        Ok(samples)
    }

    /// Preprocess text for better TTS synthesis of long paragraphs
    fn preprocess_text_for_tts(&self, text: &str) -> String {
        let mut processed = text.to_string();

        // Clean up excessive whitespace
        processed = processed
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // Add small pauses after sentences for better listening experience
        // Piper handles sentence boundaries automatically, but we can ensure clean text
        processed = processed
            .replace("  ", " ") // Remove double spaces
            .replace(" ,", ",") // Fix spacing around commas
            .replace(" .", ".") // Fix spacing around periods
            .replace(" !", "!") // Fix spacing around exclamation marks
            .replace(" ?", "?") // Fix spacing around question marks
            .trim()
            .to_string();

        // Piper can handle very long text well, but for optimal performance and user experience,
        // we'll limit to reasonable lengths while preserving sentence boundaries
        if processed.len() > 15000 {
            processed = processed.chars().take(15000).collect();
            // Try to end at a sentence boundary for better listening experience
            if let Some(last_sentence_end) =
                processed.rfind(|c: char| c == '.' || c == '!' || c == '?')
            {
                if last_sentence_end > processed.len() / 2 {
                    processed = processed.chars().take(last_sentence_end + 1).collect();
                }
            }
            tracing::info!(
                "Long text truncated to {} characters for optimal TTS performance",
                processed.len()
            );
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
        reader
            .read_exact(&mut header)
            .map_err(|e| Error::Infrastructure(format!("Failed to read WAV header: {}", e)))?;

        // Verify RIFF header
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(Error::Infrastructure("Invalid WAV file format".to_string()));
        }

        // Find data chunk (may not be at offset 44 for all WAV files)
        let mut data_start = 12;
        loop {
            reader
                .seek(SeekFrom::Start(data_start as u64))
                .map_err(|e| Error::Infrastructure(format!("Failed to seek in WAV: {}", e)))?;

            let mut chunk_header = [0u8; 8];
            if reader.read_exact(&mut chunk_header).is_err() {
                break;
            }

            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]);

            if chunk_id == b"data" {
                // Read PCM data
                let mut data = vec![0u8; chunk_size as usize];
                reader.read_exact(&mut data).map_err(|e| {
                    Error::Infrastructure(format!("Failed to read PCM data: {}", e))
                })?;

                // Convert bytes to i16 samples (assuming 16-bit little-endian)
                let samples: Vec<i16> = data
                    .chunks_exact(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();

                return Ok(samples);
            }

            data_start += 8 + chunk_size as usize;
        }

        Err(Error::Infrastructure(
            "No data chunk found in WAV file".to_string(),
        ))
    }
}

#[async_trait]
impl TextToSpeechService for TtsAdapter {
    async fn synthesize(&self, text: &str, _voice: Option<&str>) -> Result<Vec<i16>> {
        tracing::info!("Synthesizing text: '{}' using Piper Amy voice model", text);

        // Piper TTS is the only TTS engine supported - no fallbacks
        self.synthesize_piper(text).await
    }

    async fn get_available_voices(&self) -> Result<Vec<String>> {
        // Only Amy voice model is available
        let voices = vec!["default".to_string(), "amy".to_string()];

        Ok(voices)
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!(
            "TTS adapter initialized - Piper Amy voice model ready, sample_rate: {}",
            self.sample_rate
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("TTS adapter shutdown");
        Ok(())
    }
}
