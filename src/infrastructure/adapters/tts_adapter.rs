use crate::domain::services::TextToSpeechService;
use crate::shared::{Error, Result};
use async_trait::async_trait;
use std::process::Command;

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
    use_espeak: bool,
    use_festival: bool,
}

impl TtsAdapter {
    pub fn new() -> Result<Self> {
        // Check for available TTS engines
        let use_espeak = Command::new("which")
            .arg("espeak-ng")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("which")
                .arg("espeak")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        let use_festival = Command::new("which")
            .arg("festival")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if use_espeak {
            tracing::info!("TTS: espeak/espeak-ng detected");
        }
        if use_festival {
            tracing::info!("TTS: festival detected");
        }
        if !use_espeak && !use_festival {
            tracing::warn!("TTS: No TTS engine found, using tone generation fallback");
        }

        Ok(Self {
            sample_rate: 44100,
            default_voice: VoiceConfig::default(),
            use_espeak,
            use_festival,
        })
    }

    /// Generate speech using espeak-ng
    async fn synthesize_espeak(&self, text: &str, voice: &VoiceConfig) -> Result<Vec<i16>> {
        use std::io::Read;

        // espeak-ng can output raw audio to stdout
        let espeak_cmd = if Command::new("which")
            .arg("espeak-ng")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "espeak-ng"
        } else {
            "espeak"
        };

        // Create temporary file for WAV output
        let temp_path = format!("/tmp/vibespeak_tts_{}.wav", uuid::Uuid::new_v4());

        let rate = (175.0 * voice.rate) as u32; // espeak default is 175 wpm
        let pitch = (50.0 * voice.pitch) as u32; // espeak pitch: 0-99, default 50
        let amplitude = (100.0 * voice.volume) as u32;

        let output = Command::new(espeak_cmd)
            .args([
                "-w", &temp_path,
                "-s", &rate.to_string(),
                "-p", &pitch.to_string(),
                "-a", &amplitude.to_string(),
                text,
            ])
            .output()
            .map_err(|e| Error::Infrastructure(format!("Failed to run espeak: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Infrastructure(format!("espeak failed: {}", stderr)));
        }

        // Read the WAV file and extract PCM data
        let samples = self.read_wav_file(&temp_path)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(samples)
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

    /// Generate a tone-based speech simulation (fallback)
    fn generate_tone_speech(&self, text: &str, voice: &VoiceConfig) -> Vec<i16> {
        let chars = text.chars().count();
        let duration_per_char = 0.08 * (1.0 / voice.rate); // seconds per character
        let total_duration = (chars as f32 * duration_per_char).max(0.3);
        let duration_samples = (self.sample_rate as f32 * total_duration) as usize;

        let mut samples = Vec::with_capacity(duration_samples);
        let base_freq = 200.0 * voice.pitch;

        for i in 0..duration_samples {
            let t = i as f32 / self.sample_rate as f32;

            // Create varying frequency based on position (simulates speech melody)
            let position = i as f32 / duration_samples as f32;
            let freq_variation = (position * 3.0 * std::f32::consts::PI).sin() * 50.0;
            let frequency = base_freq + freq_variation;

            // Add some harmonics for richer sound
            let fundamental = (t * frequency * 2.0 * std::f32::consts::PI).sin();
            let harmonic2 = (t * frequency * 2.0 * 2.0 * std::f32::consts::PI).sin() * 0.3;
            let harmonic3 = (t * frequency * 3.0 * 2.0 * std::f32::consts::PI).sin() * 0.15;

            let sample = fundamental + harmonic2 + harmonic3;

            // Apply envelope (attack, sustain, release)
            let envelope = if position < 0.1 {
                position * 10.0 // Attack
            } else if position > 0.9 {
                (1.0 - position) * 10.0 // Release
            } else {
                1.0 // Sustain
            };

            samples.push((sample * envelope * voice.volume * i16::MAX as f32 * 0.3) as i16);
        }

        samples
    }
}

#[async_trait]
impl TextToSpeechService for TtsAdapter {
    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<Vec<i16>> {
        tracing::info!("Synthesizing text: '{}' with voice: {:?}", text, voice);

        let voice_config = match voice {
            Some("male") => VoiceConfig {
                name: "male".to_string(),
                pitch: 0.8,
                rate: 1.0,
                volume: 0.8,
            },
            Some("female") => VoiceConfig {
                name: "female".to_string(),
                pitch: 1.3,
                rate: 1.0,
                volume: 0.8,
            },
            Some("fast") => VoiceConfig {
                name: "fast".to_string(),
                pitch: 1.0,
                rate: 1.5,
                volume: 0.8,
            },
            Some("slow") => VoiceConfig {
                name: "slow".to_string(),
                pitch: 1.0,
                rate: 0.7,
                volume: 0.8,
            },
            _ => self.default_voice.clone(),
        };

        // Try espeak first if available
        if self.use_espeak {
            match self.synthesize_espeak(text, &voice_config).await {
                Ok(samples) => return Ok(samples),
                Err(e) => {
                    tracing::warn!("espeak synthesis failed, falling back to tone: {}", e);
                }
            }
        }

        // Fallback to tone generation
        Ok(self.generate_tone_speech(text, &voice_config))
    }

    async fn get_available_voices(&self) -> Result<Vec<String>> {
        let mut voices = vec![
            "default".to_string(),
            "male".to_string(),
            "female".to_string(),
            "fast".to_string(),
            "slow".to_string(),
        ];

        // If espeak is available, we could list its voices too
        if self.use_espeak {
            voices.push("espeak".to_string());
        }

        Ok(voices)
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!(
            "TTS adapter initialized - espeak: {}, festival: {}, sample_rate: {}",
            self.use_espeak,
            self.use_festival,
            self.sample_rate
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("TTS adapter shutdown");
        Ok(())
    }
}
