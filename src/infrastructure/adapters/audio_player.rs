use crate::shared::{Error, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

/// Information about an audio device
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
}

/// Audio player using cpal for cross-platform audio playback
pub struct AudioPlayer {
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    is_playing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
}

// Safety: cpal::Device is Send+Sync on supported platforms and we only share atomic state across threads.
unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    /// Create a new audio player with the default output device
    pub fn new() -> Result<Self> {
        Self::with_device(None)
    }

    /// Create a new audio player with a specific output device
    pub fn with_device(device_name: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => {
                let devices = host.output_devices()
                    .map_err(|e| Error::Audio(format!("Failed to enumerate output devices: {}", e)))?;

                let mut selected_device = None;
                for device in devices {
                    if let Ok(dev_name) = device.name() {
                        if dev_name.contains(name) {
                            selected_device = Some(device);
                            break;
                        }
                    }
                }

                selected_device.ok_or_else(|| Error::Audio(format!("Output device '{}' not found", name)))?
            }
            None => host
                .default_output_device()
                .ok_or_else(|| Error::Audio("No output device available".to_string()))?,
        };

        let config = device
            .default_output_config()
            .map_err(|e| Error::Audio(format!("Failed to get default output config: {}", e)))?;

        let device_name = device
            .name()
            .unwrap_or_else(|_| "Unknown device".to_string());

        tracing::info!(
            "Audio player initialized with output device: {} (sample_rate: {}, channels: {})",
            device_name,
            config.sample_rate().0,
            config.channels()
        );

        Ok(Self {
            device,
            config,
            is_playing: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// List all available output devices
    pub fn list_output_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = host
            .output_devices()
            .map_err(|e| Error::Audio(format!("Failed to enumerate output devices: {}", e)))?;

        let device_names: Vec<String> = devices
            .filter_map(|d| d.name().ok())
            .collect();

        Ok(device_names)
    }

    /// Pick a SupportedStreamConfig matching requested sample rate if possible, else fallback to default.
    fn select_config(&self, sample_rate: u32) -> Result<cpal::SupportedStreamConfig> {
        if sample_rate == self.config.sample_rate().0 {
            return Ok(self.config.clone());
        }

        let supported_configs = self
            .device
            .supported_output_configs()
            .map_err(|e| Error::Audio(format!("Failed to get supported configs: {}", e)))?;

        let target = cpal::SampleRate(sample_rate);

        for cfg in supported_configs {
            if cfg.channels() >= 1
                && cfg.min_sample_rate() <= target
                && cfg.max_sample_rate() >= target
            {
                return Ok(cfg.with_sample_rate(target));
            }
        }

        Err(Error::Audio(format!(
            "Sample rate {} Hz not supported",
            sample_rate
        )))
    }

    /// Play PCM audio data (i16 mono). This awaits until playback finishes or stop() is called.
    pub async fn play_pcm_data(&self, data: &[i16], sample_rate: u32) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        // Convert i16 samples to f32 in [-1.0, 1.0]
        let samples_f32: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

        let device = self.device.clone();
        let cfg = self.select_config(sample_rate)?;
        let stream_cfg: cpal::StreamConfig = cfg.clone().into();

        let channels = stream_cfg.channels as usize;

        let is_playing = self.is_playing.clone();
        let stop_flag = self.stop_flag.clone();

        // Shared playback position (in frames = mono sample index)
        let samples = Arc::new(samples_f32);
        let position = Arc::new(Mutex::new(0usize));

        tokio::task::spawn_blocking(move || -> Result<()> {
            let err_fn = |err| tracing::error!("Audio stream error: {}", err);

            match cfg.sample_format() {
                cpal::SampleFormat::F32 => {
                    let samples = samples.clone();
                    let position = position.clone();
                    let is_playing = is_playing.clone();
                    let stop_flag = stop_flag.clone();

                    let stream = device
                        .build_output_stream(
                            &stream_cfg,
                            {
                                let is_playing_clone = is_playing.clone();
                                let stop_flag_clone = stop_flag.clone();
                                move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    if stop_flag_clone.load(Ordering::SeqCst) {
                                        // Fill silence and end
                                        for v in output.iter_mut() {
                                            *v = 0.0;
                                        }
                                        is_playing_clone.store(false, Ordering::SeqCst);
                                        return;
                                    }

                                    let mut pos = position.lock().unwrap();

                                    // output is interleaved: frames * channels
                                    let frames = output.len() / channels;

                                    for frame in 0..frames {
                                        let sample = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = sample;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| {
                            Error::Audio(format!("Failed to build output stream: {}", e))
                        })?;

                    stream
                        .play()
                        .map_err(|e| Error::Audio(format!("Failed to start playback: {}", e)))?;

                    // Keep stream alive until done or stopped
                    while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(20));
                    }

                    drop(stream);
                    Ok(())
                }

                cpal::SampleFormat::I16 => {
                    let samples = samples.clone();
                    let position = position.clone();
                    let is_playing = is_playing.clone();
                    let stop_flag = stop_flag.clone();

                    let stream = device
                        .build_output_stream(
                            &stream_cfg,
                            {
                                let is_playing_clone = is_playing.clone();
                                let stop_flag_clone = stop_flag.clone();
                                move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                    if stop_flag_clone.load(Ordering::SeqCst) {
                                        for v in output.iter_mut() {
                                            *v = 0;
                                        }
                                        is_playing_clone.store(false, Ordering::SeqCst);
                                        return;
                                    }

                                    let mut pos = position.lock().unwrap();
                                    let frames = output.len() / channels;

                                    for frame in 0..frames {
                                        let sample_f32 = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        let sample_i16 =
                                            (sample_f32.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = sample_i16;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| {
                            Error::Audio(format!("Failed to build output stream: {}", e))
                        })?;

                    stream
                        .play()
                        .map_err(|e| Error::Audio(format!("Failed to start playback: {}", e)))?;

                    while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(20));
                    }

                    drop(stream);
                    Ok(())
                }

                cpal::SampleFormat::U16 => {
                    let samples = samples.clone();
                    let position = position.clone();
                    let is_playing = is_playing.clone();
                    let stop_flag = stop_flag.clone();

                    let stream = device
                        .build_output_stream(
                            &stream_cfg,
                            {
                                let is_playing_clone = is_playing.clone();
                                let stop_flag_clone = stop_flag.clone();
                                move |output: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                    if stop_flag_clone.load(Ordering::SeqCst) {
                                        for v in output.iter_mut() {
                                            *v = u16::MAX / 2;
                                        }
                                        is_playing_clone.store(false, Ordering::SeqCst);
                                        return;
                                    }

                                    let mut pos = position.lock().unwrap();
                                    let frames = output.len() / channels;

                                    for frame in 0..frames {
                                        let sample_f32 = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        // map [-1,1] -> [0, u16::MAX]
                                        let u = ((sample_f32.clamp(-1.0, 1.0) + 1.0)
                                            * 0.5
                                            * u16::MAX as f32)
                                            as u16;

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = u;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| {
                            Error::Audio(format!("Failed to build output stream: {}", e))
                        })?;

                    stream
                        .play()
                        .map_err(|e| Error::Audio(format!("Failed to start playback: {}", e)))?;

                    while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(20));
                    }

                    drop(stream);
                    Ok(())
                }
                other => Err(Error::Audio(format!(
                    "Unsupported sample format: {:?}",
                    other
                ))),
            }
        })
        .await
        .map_err(|e| Error::Audio(format!("Playback task failed: {}", e)))??;

        Ok(())
    }

    /// Play audio non-blocking (fire and forget)
    pub fn play_pcm_data_nonblocking(&self, data: Vec<i16>, sample_rate: u32) {
        if data.is_empty() {
            return;
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        let device = self.device.clone();
        let cfg = match self.select_config(sample_rate) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to select config: {}", e);
                self.is_playing.store(false, Ordering::SeqCst);
                return;
            }
        };
        let stream_cfg: cpal::StreamConfig = cfg.clone().into();
        let channels = stream_cfg.channels as usize;

        let samples_f32: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

        let samples = Arc::new(samples_f32);
        let position = Arc::new(Mutex::new(0usize));

        let is_playing = self.is_playing.clone();
        let stop_flag = self.stop_flag.clone();

        std::thread::spawn(move || {
            let err_fn = |err| tracing::error!("Audio stream error: {}", err);

            let run = || -> Result<()> {
                match cfg.sample_format() {
                    cpal::SampleFormat::F32 => {
                        let samples = samples.clone();
                        let position = position.clone();
                        let is_playing = is_playing.clone();
                        let stop_flag = stop_flag.clone();

                        let stream = device
                            .build_output_stream(
                                &stream_cfg,
                                {
                                    let is_playing_clone = is_playing.clone();
                                    let stop_flag_clone = stop_flag.clone();
                                    move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                        if stop_flag_clone.load(Ordering::SeqCst) {
                                            for v in output.iter_mut() {
                                                *v = 0.0;
                                            }
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            return;
                                        }

                                    let mut pos = position.lock().unwrap();
                                    let frames = output.len() / channels;

                                    for frame in 0..frames {
                                        let sample = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = sample;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                            )
                            .map_err(|e| {
                                Error::Audio(format!("Failed to build output stream: {}", e))
                            })?;

                        stream.play().map_err(|e| {
                            Error::Audio(format!("Failed to start playback: {}", e))
                        })?;

                        while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst)
                        {
                            std::thread::sleep(Duration::from_millis(20));
                        }

                        drop(stream);
                        Ok(())
                    }

                    cpal::SampleFormat::I16 => {
                        let samples = samples.clone();
                        let position = position.clone();
                        let is_playing = is_playing.clone();
                        let stop_flag = stop_flag.clone();

                        let stream = device
                            .build_output_stream(
                                &stream_cfg,
                                {
                                    let is_playing_clone = is_playing.clone();
                                    let stop_flag_clone = stop_flag.clone();
                                    move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                        if stop_flag_clone.load(Ordering::SeqCst) {
                                            for v in output.iter_mut() {
                                                *v = 0;
                                            }
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            return;
                                        }

                                    let mut pos = position.lock().unwrap();
                                    let frames = output.len() / channels;

                                    for frame in 0..frames {
                                        let sample_f32 = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        let sample_i16 =
                                            (sample_f32.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = sample_i16;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                            )
                            .map_err(|e| {
                                Error::Audio(format!("Failed to build output stream: {}", e))
                            })?;

                        stream.play().map_err(|e| {
                            Error::Audio(format!("Failed to start playback: {}", e))
                        })?;

                        while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst)
                        {
                            std::thread::sleep(Duration::from_millis(20));
                        }

                        drop(stream);
                        Ok(())
                    }

                    cpal::SampleFormat::U16 => {
                        let samples = samples.clone();
                        let position = position.clone();
                        let is_playing = is_playing.clone();
                        let stop_flag = stop_flag.clone();

                        let stream = device
                            .build_output_stream(
                                &stream_cfg,
                                {
                                    let is_playing_clone = is_playing.clone();
                                    let stop_flag_clone = stop_flag.clone();
                                    move |output: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                        if stop_flag_clone.load(Ordering::SeqCst) {
                                            for v in output.iter_mut() {
                                                *v = u16::MAX / 2;
                                            }
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            return;
                                        }

                                        let mut pos = position.lock().unwrap();
                                        let frames = output.len() / channels;

                                        for frame in 0..frames {
                                        let sample_f32 = if *pos < samples.len() {
                                            let s = samples[*pos];
                                            *pos = *pos + 1;
                                            s
                                        } else {
                                            is_playing_clone.store(false, Ordering::SeqCst);
                                            0.0
                                        };

                                        let u = ((sample_f32.clamp(-1.0, 1.0) + 1.0)
                                            * 0.5
                                            * u16::MAX as f32)
                                            as u16;

                                        let base = frame * channels;
                                        for ch in 0..channels {
                                            output[base + ch] = u;
                                        }
                                    }
                                }
                            },
                            err_fn,
                            None,
                            )
                            .map_err(|e| {
                                Error::Audio(format!("Failed to build output stream: {}", e))
                            })?;

                        stream.play().map_err(|e| {
                            Error::Audio(format!("Failed to start playback: {}", e))
                        })?;

                        while is_playing.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst)
                        {
                            std::thread::sleep(Duration::from_millis(20));
                        }

                        drop(stream);
                        Ok(())
                    }

                    other => Err(Error::Audio(format!(
                        "Unsupported sample format: {:?}",
                        other
                    ))),
                }
            };

            if let Err(e) = run() {
                tracing::error!("Nonblocking playback error: {}", e);
                is_playing.store(false, Ordering::SeqCst);
            }
        });
    }

    /// Play audio from a file
    pub async fn play_file(&self, _path: &str) -> Result<()> {
        Err(Error::Audio(
            "File playback not yet implemented with CPAL. Use play_pcm_data instead.".to_string(),
        ))
    }

    /// Stop current playback
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        tracing::debug!("Audio playback stop requested");
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    /// Get information about the current audio device
    pub fn device_info(&self) -> Result<AudioDeviceInfo> {
        let name = self.device
            .name()
            .unwrap_or_else(|_| "Unknown device".to_string());

        Ok(AudioDeviceInfo {
            name,
            sample_rate: self.config.sample_rate().0,
            channels: self.config.channels(),
            sample_format: format!("{:?}", self.config.sample_format()),
        })
    }

    /// Attempt to reinitialize with a different device if the current one fails
    pub fn try_reinitialize_with_fallback(&mut self) -> Result<()> {
        tracing::warn!("Attempting to reinitialize audio player with fallback device");

        // Try to find another available output device
        if let Ok(devices) = Self::list_output_devices() {
            for device_name in devices {
                if let Ok(new_player) = Self::with_device(Some(&device_name)) {
                    *self = new_player;
                    tracing::info!("Successfully reinitialized with device: {}", device_name);
                    return Ok(());
                }
            }
        }

        // If no other devices work, try default
        if let Ok(new_player) = Self::new() {
            *self = new_player;
            tracing::info!("Reinitialized with default device");
            Ok(())
        } else {
            Err(Error::Audio("Failed to reinitialize audio player with any device".to_string()))
        }
    }
}
