use crate::shared::{Result, Error};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

/// Audio player using rodio for cross-platform audio playback
pub struct AudioPlayer {
    stream_handle: Arc<OutputStreamHandle>,
    _stream: OutputStream,
    is_playing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
}

// Safety: AudioPlayer is Send+Sync because we use Arc for shared state
unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| Error::Audio(format!("Failed to initialize audio output: {}", e)))?;

        tracing::info!("Audio player initialized with default output device");

        Ok(Self {
            _stream: stream,
            stream_handle: Arc::new(stream_handle),
            is_playing: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Play PCM audio data
    pub async fn play_pcm_data(&self, data: &[i16], sample_rate: u32) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        // Convert i16 samples to f32 (owned data for the closure)
        let samples: Vec<f32> = data.iter()
            .map(|&s| s as f32 / i16::MAX as f32)
            .collect();
        let sample_count = samples.len();

        // Create a source from the samples
        let source = PcmSource::new(samples, sample_rate, 1);

        let stream_handle = self.stream_handle.clone();
        let is_playing = self.is_playing.clone();
        let stop_flag = self.stop_flag.clone();

        tokio::task::spawn_blocking(move || {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                sink.append(source);
                tracing::debug!("Playing {} samples at {} Hz", sample_count, sample_rate);

                // Poll for completion or stop signal
                while !sink.empty() && !stop_flag.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(50));
                }

                if stop_flag.load(Ordering::SeqCst) {
                    sink.stop();
                }
            }
            is_playing.store(false, Ordering::SeqCst);
        }).await
            .map_err(|e| Error::Audio(format!("Playback task failed: {}", e)))?;

        Ok(())
    }

    /// Play audio non-blocking (fire and forget)
    pub fn play_pcm_data_nonblocking(&self, data: Vec<i16>, sample_rate: u32) {
        if data.is_empty() {
            return;
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        let samples: Vec<f32> = data.iter()
            .map(|&s| s as f32 / i16::MAX as f32)
            .collect();

        let source = PcmSource::new(samples, sample_rate, 1);
        let stream_handle = self.stream_handle.clone();
        let is_playing = self.is_playing.clone();
        let stop_flag = self.stop_flag.clone();

        std::thread::spawn(move || {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                sink.append(source);

                while !sink.empty() && !stop_flag.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(50));
                }

                if stop_flag.load(Ordering::SeqCst) {
                    sink.stop();
                }
            }
            is_playing.store(false, Ordering::SeqCst);
        });
    }

    /// Play audio from a file
    pub async fn play_file(&self, path: &str) -> Result<()> {
        let path = path.to_string();
        let stream_handle = self.stream_handle.clone();
        let is_playing = self.is_playing.clone();
        let stop_flag = self.stop_flag.clone();

        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&path)
                .map_err(|e| Error::Audio(format!("Failed to open audio file: {}", e)))?;

            let source = rodio::Decoder::new(std::io::BufReader::new(file))
                .map_err(|e| Error::Audio(format!("Failed to decode audio file: {}", e)))?;

            if let Ok(sink) = Sink::try_new(&stream_handle) {
                sink.append(source);
                tracing::info!("Playing audio file: {}", path);

                while !sink.empty() && !stop_flag.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(50));
                }

                if stop_flag.load(Ordering::SeqCst) {
                    sink.stop();
                }
            }
            is_playing.store(false, Ordering::SeqCst);
            Ok(())
        }).await
            .map_err(|e| Error::Audio(format!("Playback task failed: {}", e)))??;

        Ok(())
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
}

/// Custom PCM source for rodio
struct PcmSource {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    position: usize,
}

impl PcmSource {
    fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            position: 0,
        }
    }
}

impl Iterator for PcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for PcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let samples = self.samples.len() as u64;
        let duration_secs = samples / (self.sample_rate as u64 * self.channels as u64).max(1);
        Some(Duration::from_secs(duration_secs))
    }
}