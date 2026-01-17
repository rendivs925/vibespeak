use crate::shared::{AudioSample, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionSession {
    pub id: SessionId,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub audio_samples: Vec<AudioSample>,
    pub recognition_results: Vec<RecognitionResult>,
    pub status: SessionStatus,
    pub metadata: SessionMetadata,
}

impl RecognitionSession {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            audio_samples: Vec::new(),
            recognition_results: Vec::new(),
            status: SessionStatus::Active,
            metadata: SessionMetadata::default(),
        }
    }

    pub fn add_audio_sample(&mut self, sample: AudioSample) {
        self.audio_samples.push(sample);
    }

    pub fn add_recognition_result(&mut self, result: RecognitionResult) {
        self.recognition_results.push(result);
    }

    pub fn end_session(&mut self) {
        self.end_time = Some(chrono::Utc::now());
        self.status = SessionStatus::Completed;
    }

    pub fn get_duration(&self) -> Option<std::time::Duration> {
        self.end_time.map(|end| {
            (end - self.start_time)
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(0))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
    pub text: String,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub alternatives: Vec<String>,
}

impl RecognitionResult {
    pub fn new(text: String, confidence: f64) -> Self {
        Self {
            text,
            confidence,
            timestamp: chrono::Utc::now(),
            alternatives: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub user_id: Option<String>,
    pub device_info: HashMap<String, String>,
    pub environment: HashMap<String, String>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            user_id: None,
            device_info: HashMap::new(),
            environment: HashMap::new(),
        }
    }
}
