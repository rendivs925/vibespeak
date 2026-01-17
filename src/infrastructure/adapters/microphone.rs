// Microphone capture adapter using CPAL for cross-platform audio input

use crate::shared::{AudioSample, Error, Result};

/// Information about an audio device
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
}
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Configuration for microphone capture
#[derive(Debug, Clone)]
pub struct MicrophoneConfig {
    /// Sample rate in Hz (default: 16000 for speech recognition)
    pub sample_rate: u32,
    /// Number of channels (default: 1 for mono)
    pub channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
    /// Silence threshold for voice activity detection
    pub silence_threshold: f32,
    /// Minimum speech duration in milliseconds
    pub min_speech_ms: u32,
    /// Maximum speech duration in milliseconds
    pub max_speech_ms: u32,
    /// Silence duration to end recording in milliseconds
    pub silence_end_ms: u32,
}

impl Default for MicrophoneConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            silence_threshold: 0.01,
            min_speech_ms: 500,
            max_speech_ms: 30000,
            silence_end_ms: 1000,
        }
    }
}

impl MicrophoneConfig {
    /// Configuration optimized for voice commands (fast response, shorter silence timeout)
    pub fn for_voice_commands() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 512,
            silence_threshold: 0.015,
            min_speech_ms: 300,
            max_speech_ms: 10000,
            silence_end_ms: 800,
        }
    }

    /// Configuration optimized for speech recognition (higher quality, longer timeouts)
    pub fn for_speech_recognition() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            silence_threshold: 0.025, // Increased from 0.01 to be more sensitive to normal speech
            min_speech_ms: 300, // Reduced from 500ms to catch shorter commands
            max_speech_ms: 45000,
            silence_end_ms: 1200, // Reduced from 1500ms for faster response
        }
    }

    /// Configuration optimized for recording music or high-quality audio
    pub fn for_high_quality_recording() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 2048,
            silence_threshold: 0.005,
            min_speech_ms: 1000,
            max_speech_ms: 300000, // 5 minutes
            silence_end_ms: 2000,
        }
    }

    /// Configuration optimized for low-latency applications
    pub fn for_low_latency() -> Self {
        Self {
            sample_rate: 22050,
            channels: 1,
            buffer_size: 256,
            silence_threshold: 0.02,
            min_speech_ms: 200,
            max_speech_ms: 15000,
            silence_end_ms: 500,
        }
    }
}

/// Microphone capture adapter for recording audio from the system's default input device
pub struct MicrophoneCapture {
    config: MicrophoneConfig,
    is_recording: Arc<AtomicBool>,
    device_name: Option<String>,
}

impl MicrophoneCapture {
    /// Create a new microphone capture instance with default configuration and device
    pub fn new() -> Result<Self> {
        Self::with_config_and_device(MicrophoneConfig::default(), None)
    }

    /// Create a new microphone capture instance with custom configuration
    pub fn with_config(config: MicrophoneConfig) -> Result<Self> {
        Self::with_config_and_device(config, None)
    }

    /// Create a new microphone capture instance with custom configuration and specific device
    pub fn with_config_and_device(config: MicrophoneConfig, device_name: Option<&str>) -> Result<Self> {
        // Verify audio host is available
        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => {
                let devices = host.input_devices()
                    .map_err(|e| Error::Audio(format!("Failed to enumerate input devices: {}", e)))?;

                let mut selected_device = None;
                for device in devices {
                    if let Ok(dev_name) = device.name() {
                        if dev_name.contains(name) {
                            selected_device = Some(device);
                            break;
                        }
                    }
                }

                selected_device.ok_or_else(|| Error::Audio(format!("Input device '{}' not found", name)))?
            }
            None => host.default_input_device()
                .ok_or_else(|| Error::Audio("No input device available".to_string()))?,
        };

        let device_name = device.name().ok();

        tracing::info!(
            "Microphone initialized: {:?} (sample_rate: {}, channels: {})",
            device_name,
            config.sample_rate,
            config.channels
        );

        Ok(Self {
            config,
            is_recording: Arc::new(AtomicBool::new(false)),
            device_name,
        })
    }

    /// Get the name of the current input device
    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    /// List all available input devices
    pub fn list_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices: Vec<String> = host
            .input_devices()
            .map_err(|e| Error::Audio(format!("Failed to enumerate input devices: {}", e)))?
            .filter_map(|d| d.name().ok())
            .collect();

        Ok(devices)
    }

    fn get_input_device(&self) -> Result<cpal::Device> {
        let host = cpal::default_host();
        if let Some(name) = &self.device_name {
            let devices = host
                .input_devices()
                .map_err(|e| Error::Audio(format!("Failed to enumerate input devices: {}", e)))?;
            for device in devices {
                if let Ok(dev_name) = device.name() {
                    if dev_name.contains(name) {
                        return Ok(device);
                    }
                }
            }
            Err(Error::Audio(format!("Input device '{}' not found", name)))
        } else {
            host.default_input_device()
                .ok_or_else(|| Error::Audio("No input device available".to_string()))
        }
    }

    fn downmix_to_mono<'a>(data: &'a [i16], channels: u16) -> Cow<'a, [i16]> {
        if channels <= 1 {
            return Cow::Borrowed(data);
        }

        let mut mono = Vec::with_capacity(data.len() / channels as usize);
        for frame in data.chunks(channels as usize) {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            mono.push((sum / frame.len() as i32) as i16);
        }

        Cow::Owned(mono)
    }

    fn get_candidate_configs(&self, device: &cpal::Device) -> Result<Vec<cpal::SupportedStreamConfig>> {
        let desired_rate = cpal::SampleRate(self.config.sample_rate);
        let desired_channels = self.config.channels;
        let mut candidates = Vec::new();

        let ranges: Vec<_> = device
            .supported_input_configs()
            .map_err(|e| Error::Audio(format!("Failed to get supported configs: {}", e)))?
            .collect();

        for range in &ranges {
            if range.channels() >= desired_channels
                && range.min_sample_rate() <= desired_rate
                && range.max_sample_rate() >= desired_rate
            {
                candidates.push(range.clone().with_sample_rate(desired_rate));
                break;
            }
        }

        if let Ok(default_config) = device.default_input_config() {
            if !candidates.iter().any(|c| {
                c.channels() == default_config.channels()
                    && c.sample_rate() == default_config.sample_rate()
                    && c.sample_format() == default_config.sample_format()
            }) {
                candidates.push(default_config);
            }
        }

        for range in ranges {
            if range.channels() < desired_channels {
                continue;
            }
            let candidate = range.with_sample_rate(range.max_sample_rate());
            if !candidates.iter().any(|c| {
                c.channels() == candidate.channels()
                    && c.sample_rate() == candidate.sample_rate()
                    && c.sample_format() == candidate.sample_format()
            }) {
                candidates.push(candidate);
            }
        }

        if candidates.is_empty() {
            return Err(Error::Audio("No supported input configurations found".to_string()));
        }

        Ok(candidates)
    }

    fn build_input_stream_i16<F>(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        mut on_samples: F,
    ) -> Result<cpal::Stream>
    where
        F: FnMut(&[i16]) + Send + 'static,
    {
        let err_fn = |err| tracing::error!("Audio stream error: {}", err);
        match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let samples: Vec<i16> = data
                        .iter()
                        .map(|&sample| (sample * i16::MAX as f32) as i16)
                        .collect();
                    on_samples(&samples);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    on_samples(data);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let samples: Vec<i16> = data
                        .iter()
                        .map(|&sample| (sample as i32 - 32768) as i16)
                        .collect();
                    on_samples(&samples);
                },
                err_fn,
                None,
            ),
            _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
        }
        .map_err(|e| Error::Audio(format!("Failed to build input stream: {}", e)))
    }

    fn build_input_stream_i16_with_fallback<F>(
        &self,
        device: &cpal::Device,
        on_samples: F,
    ) -> Result<(cpal::Stream, cpal::SupportedStreamConfig)>
    where
        F: FnMut(&[i16], u16) + Send + 'static,
    {
        let on_samples = Arc::new(Mutex::new(on_samples));
        let candidates = self.get_candidate_configs(device)?;
        for config in candidates {
            let channels = config.channels();
            let handler = on_samples.clone();
            match self.build_input_stream_i16(device, &config, move |data| {
                if let Ok(mut on_samples) = handler.lock() {
                    on_samples(data, channels);
                }
            }) {
                Ok(stream) => return Ok((stream, config)),
                Err(err) => {
                    tracing::warn!(
                        "Input config rejected (channels: {}, rate: {}, format: {:?}): {}",
                        config.channels(),
                        config.sample_rate().0,
                        config.sample_format(),
                        err
                    );
                }
            }
        }

        Err(Error::Audio(
            "Failed to build input stream with any supported config".to_string(),
        ))
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Stop the current recording
    pub fn stop_recording(&self) {
        self.is_recording.store(false, Ordering::SeqCst);
    }

    /// Get information about the current microphone device
    pub fn device_info(&self) -> Result<AudioDeviceInfo> {
        let name = self.device_name.clone()
            .unwrap_or_else(|| "Unknown device".to_string());

        // We can't easily get the actual config without creating a device,
        // so we'll provide estimated info based on our configuration
        Ok(AudioDeviceInfo {
            name,
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            sample_format: "i16".to_string(), // We convert to i16
        })
    }

    /// Attempt to reinitialize with a different device if the current one fails
    pub fn try_reinitialize_with_fallback(&mut self) -> Result<()> {
        tracing::warn!("Attempting to reinitialize microphone with fallback device");

        // Try to find another available input device
        if let Ok(devices) = Self::list_devices() {
            for device_name in devices {
                if let Ok(new_mic) = Self::with_config_and_device(self.config.clone(), Some(&device_name)) {
                    *self = new_mic;
                    tracing::info!("Successfully reinitialized microphone with device: {}", device_name);
                    return Ok(());
                }
            }
        }

        // If no other devices work, try default
        if let Ok(new_mic) = Self::with_config(self.config.clone()) {
            *self = new_mic;
            tracing::info!("Reinitialized microphone with default device");
            Ok(())
        } else {
            Err(Error::Audio("Failed to reinitialize microphone with any device".to_string()))
        }
    }

    /// Record audio until silence is detected or max duration is reached
    /// Returns the recorded audio samples
    pub async fn record_until_silence(&self) -> Result<AudioSample> {
        let device = self.get_input_device()?;

        // Shared buffer for audio data
        let audio_buffer: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
        let audio_buffer_clone = audio_buffer.clone();

        // Voice activity detection state
        let silence_samples = Arc::new(Mutex::new(0usize));
        let speech_detected = Arc::new(AtomicBool::new(false));
        let silence_samples_clone = silence_samples.clone();
        let speech_detected_clone = speech_detected.clone();

        let silence_threshold = self.config.silence_threshold;
        let silence_end_samples = Arc::new(Mutex::new(0usize));
        let max_samples = Arc::new(Mutex::new(0usize));

        self.is_recording.store(true, Ordering::SeqCst);
        let is_recording = self.is_recording.clone();

        // Create the input stream
        let silence_end_samples_clone = silence_end_samples.clone();
        let max_samples_clone = max_samples.clone();
        let (stream, used_config) = self.build_input_stream_i16_with_fallback(
            &device,
            move |data: &[i16], channels: u16| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let mono = Self::downmix_to_mono(data, channels);
                let samples = mono.as_ref();

                let mut rms_sum = 0.0f32;
                for &sample in samples {
                    let normalized = sample as f32 / i16::MAX as f32;
                    rms_sum += normalized * normalized;
                }

                let rms = (rms_sum / samples.len() as f32).sqrt();
                let is_speech = rms > silence_threshold;

                if is_speech {
                    speech_detected_clone.store(true, Ordering::SeqCst);
                    *silence_samples_clone.lock().unwrap() = 0;
                } else if speech_detected_clone.load(Ordering::SeqCst) {
                    *silence_samples_clone.lock().unwrap() += samples.len();
                }

                let mut buffer = audio_buffer_clone.lock().unwrap();
                buffer.extend_from_slice(samples);

                let silence_count = *silence_samples_clone.lock().unwrap();
                if speech_detected_clone.load(Ordering::SeqCst) && silence_count >= *silence_end_samples_clone.lock().unwrap() {
                    is_recording.store(false, Ordering::SeqCst);
                }
                if buffer.len() >= *max_samples_clone.lock().unwrap() {
                    is_recording.store(false, Ordering::SeqCst);
                }
            },
        )?;

        let sample_rate = used_config.sample_rate().0;
        *silence_end_samples.lock().unwrap() =
            (self.config.silence_end_ms as f32 / 1000.0 * sample_rate as f32) as usize;
        *max_samples.lock().unwrap() =
            (self.config.max_speech_ms as f32 / 1000.0 * sample_rate as f32) as usize;

        tracing::info!(
            "Recording with config: {} Hz, {} ch, {:?}",
            sample_rate,
            used_config.channels(),
            used_config.sample_format()
        );

        // Start recording
        stream.play().map_err(|e| Error::Audio(format!("Failed to start recording: {}", e)))?;

        tracing::info!("Recording started. Speak now...");

        // Wait for recording to complete
        while self.is_recording.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        // Stop the stream
        drop(stream);

        // Get the recorded audio
        let samples = audio_buffer.lock().unwrap().clone();

        if samples.is_empty() {
            return Err(Error::Audio("No audio recorded".to_string()));
        }

        tracing::info!("Recording complete. Captured {} samples ({:.2}s)",
            samples.len(),
            samples.len() as f32 / sample_rate as f32
        );

        Ok(AudioSample::new(samples, sample_rate, 1))
    }

    /// Record for a fixed duration
    pub async fn record_duration(&self, duration_ms: u32) -> Result<AudioSample> {
        let device = self.get_input_device()?;

        let audio_buffer: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
        let audio_buffer_clone = audio_buffer.clone();

        let target_samples = Arc::new(Mutex::new(0usize));

        self.is_recording.store(true, Ordering::SeqCst);
        let is_recording = self.is_recording.clone();

        let target_samples_clone = target_samples.clone();
        let (stream, used_config) = self.build_input_stream_i16_with_fallback(
            &device,
            move |data: &[i16], channels: u16| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let mut buffer = audio_buffer_clone.lock().unwrap();
                let mono = Self::downmix_to_mono(data, channels);
                let samples = mono.as_ref();
                buffer.extend_from_slice(samples);

                if buffer.len() >= *target_samples_clone.lock().unwrap() {
                    is_recording.store(false, Ordering::SeqCst);
                }
            },
        )?;

        let sample_rate = used_config.sample_rate().0;
        *target_samples.lock().unwrap() =
            (duration_ms as f32 / 1000.0 * sample_rate as f32) as usize;

        stream.play().map_err(|e| Error::Audio(format!("Failed to start recording: {}", e)))?;

        tracing::info!("Recording for {} ms...", duration_ms);

        while self.is_recording.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        drop(stream);

        let samples = audio_buffer.lock().unwrap().clone();

        Ok(AudioSample::new(samples, sample_rate, 1))
    }

    /// Start continuous recording with a callback for each audio chunk
    pub fn start_continuous<F>(&self, mut callback: F) -> Result<ContinuousRecording>
    where
        F: FnMut(Vec<i16>) + Send + 'static,
    {
        let device = self.get_input_device()?;

        self.is_recording.store(true, Ordering::SeqCst);
        let is_recording = self.is_recording.clone();

        let (stream, used_config) = self.build_input_stream_i16_with_fallback(
            &device,
            move |data: &[i16], channels: u16| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let mono = Self::downmix_to_mono(data, channels);
                callback(mono.to_vec());
            },
        )?;

        stream.play().map_err(|e| Error::Audio(format!("Failed to start recording: {}", e)))?;

        Ok(ContinuousRecording {
            _stream: stream,
            is_recording: self.is_recording.clone(),
        })
    }

}

/// Handle for continuous recording that stops when dropped
pub struct ContinuousRecording {
    _stream: cpal::Stream,
    is_recording: Arc<AtomicBool>,
}

impl ContinuousRecording {
    /// Stop the continuous recording
    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::SeqCst);
    }
}

impl Drop for ContinuousRecording {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microphone_config_default() {
        let config = MicrophoneConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_list_devices() {
        // This test may fail in CI without audio devices
        match MicrophoneCapture::list_devices() {
            Ok(devices) => {
                println!("Available input devices: {:?}", devices);
            }
            Err(e) => {
                println!("Could not list devices (expected in CI): {}", e);
            }
        }
    }
}
