use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("WebRTC error: {0}")]
    WebRTC(String),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Infrastructure(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Infrastructure(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Configuration(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Configuration(format!("TOML deserialization error: {}", err))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Configuration(format!("TOML serialization error: {}", err))
    }
}

// Common types
pub type CommandId = String;
pub type SessionId = String;
pub type WorkflowId = String;
pub type PluginId = String;

// Audio types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSample {
    pub data: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

impl AudioSample {
    pub fn new(data: Vec<i16>, sample_rate: u32, channels: u8) -> Self {
        Self {
            data,
            sample_rate,
            channels,
        }
    }

    /// Get the duration of the audio sample in seconds
    pub fn duration_seconds(&self) -> f32 {
        self.data.len() as f32 / self.sample_rate as f32 / self.channels as f32
    }

    /// Get the number of samples per channel
    pub fn samples_per_channel(&self) -> usize {
        self.data.len() / self.channels as usize
    }

    /// Convert audio to mono by averaging channels
    pub fn to_mono(&self) -> AudioSample {
        if self.channels <= 1 {
            return self.clone();
        }

        let samples_per_channel = self.samples_per_channel();
        let mut mono_data = Vec::with_capacity(samples_per_channel);

        for frame_idx in 0..samples_per_channel {
            let mut sum: i32 = 0;
            for ch in 0..self.channels as usize {
                let sample_idx = frame_idx * self.channels as usize + ch;
                sum += self.data[sample_idx] as i32;
            }
            mono_data.push((sum / self.channels as i32) as i16);
        }

        AudioSample::new(mono_data, self.sample_rate, 1)
    }

    /// Resample audio to target sample rate
    pub fn resample(&self, target_sample_rate: u32) -> Result<AudioSample> {
        if self.sample_rate == target_sample_rate {
            return Ok(self.clone());
        }

        use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, WindowFunction};

        // Split audio into separate channels
        let samples_per_channel = self.samples_per_channel();
        let channel_data: Vec<Vec<f32>> = (0..self.channels as usize)
            .map(|ch| {
                (0..samples_per_channel)
                    .map(|frame| {
                        let sample_idx = frame * self.channels as usize + ch;
                        self.data[sample_idx] as f32 / i16::MAX as f32
                    })
                    .collect()
            })
            .collect();

        // Setup resampler
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: rubato::SincInterpolationType::Linear,
            oversampling_factor: 160,
            window: WindowFunction::BlackmanHarris2,
        };

        let mut resampler = SincFixedIn::<f32>::new(
            target_sample_rate as f64 / self.sample_rate as f64,
            2.0,
            params,
            samples_per_channel,
            self.channels as usize, // output channels = input channels
        )
        .map_err(|e| Error::Audio(format!("Failed to create resampler: {}", e)))?;

        // Pre-allocate output buffers
        let mut output_channels: Vec<Vec<f32>> = (0..self.channels as usize)
            .map(|_| vec![0.0f32; resampler.output_frames_max()])
            .collect();

        // Create slices for resampler
        let input_slices: Vec<&[f32]> = channel_data.iter().map(|ch| ch.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> = output_channels
            .iter_mut()
            .map(|ch| ch.as_mut_slice())
            .collect();

        // Resample
        let (_frames_consumed, frames_written) = resampler
            .process_into_buffer(&input_slices, &mut output_slices, None)
            .map_err(|e| Error::Audio(format!("Failed to resample audio: {}", e)))?;

        // Interleave channels back into single buffer
        let mut resampled_data = Vec::with_capacity(frames_written * self.channels as usize);
        for frame in 0..frames_written {
            for ch in 0..self.channels as usize {
                let sample = (output_channels[ch][frame] * i16::MAX as f32)
                    .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                resampled_data.push(sample);
            }
        }

        Ok(AudioSample::new(
            resampled_data,
            target_sample_rate,
            self.channels,
        ))
    }

    /// Convert to 16kHz mono (optimal for speech recognition)
    pub fn to_16khz_mono(&self) -> Result<AudioSample> {
        let mono = self.to_mono();
        mono.resample(16000)
    }

    /// Normalize the audio to prevent clipping (simple peak normalization)
    pub fn normalize(&mut self) {
        if self.data.is_empty() {
            return;
        }

        // Find the peak value
        let peak = self.data.iter().map(|&x| x.abs()).max().unwrap_or(1) as f32;

        if peak == 0.0 {
            return; // Avoid division by zero
        }

        let target_peak = (i16::MAX as f32 * 0.9) as f32; // Leave some headroom
        let scale = target_peak / peak;

        if scale < 1.0 {
            // Only normalize if we would clip
            for sample in &mut self.data {
                *sample = (*sample as f32 * scale) as i16;
            }
        }
    }

    /// Create a normalized copy of the audio sample
    pub fn normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }

    /// Apply a simple noise gate to reduce background noise
    pub fn apply_noise_gate(&mut self, threshold: f32) {
        let threshold_i16 = (threshold * i16::MAX as f32) as i16;

        for sample in &mut self.data {
            if sample.abs() < threshold_i16 {
                *sample = 0;
            }
        }
    }

    /// Get the RMS (Root Mean Square) level of the audio
    pub fn rms_level(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }

        let sum_squares: f64 = self
            .data
            .iter()
            .map(|&x| (x as f64 / i16::MAX as f64).powi(2))
            .sum();

        (sum_squares / self.data.len() as f64).sqrt() as f32
    }
}

// Security levels for script execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Sandboxed, // Restricted execution
    Trusted,   // Full access to user-approved paths
    Isolated,  // Container/VM execution
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::Sandboxed => write!(f, "sandboxed"),
            SecurityLevel::Trusted => write!(f, "trusted"),
            SecurityLevel::Isolated => write!(f, "isolated"),
        }
    }
}

// Script types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Bash,
    Python,
    JavaScript,
    Ruby,
    PowerShell,
    Custom(String),
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptType::Bash => write!(f, "bash"),
            ScriptType::Python => write!(f, "python"),
            ScriptType::JavaScript => write!(f, "javascript"),
            ScriptType::Ruby => write!(f, "ruby"),
            ScriptType::PowerShell => write!(f, "powershell"),
            ScriptType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}
