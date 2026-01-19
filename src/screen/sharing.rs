//! WebRTC screen sharing implementation

use crate::shared::{AudioSample, Error, Result};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_remote::TrackRemote;

/// Screen sharing session state
#[derive(Debug)]
pub struct ScreenSharingSession {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub session_id: String,
    pub is_active: bool,
    pub audio_receiver: Option<mpsc::UnboundedSender<AudioSample>>,
}

/// Screen sharing manager
pub struct ScreenSharingManager {
    api: webrtc::api::API,
    sessions: Arc<RwLock<std::collections::HashMap<String, ScreenSharingSession>>>,
    audio_sample_rate: u32,
}

impl ScreenSharingManager {
    pub fn new() -> Self {
        let api = APIBuilder::new().build();

        Self {
            api,
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            audio_sample_rate: 16000, // Optimal for voice recognition
        }
    }

    /// Set up audio receiver for voice recognition
    pub async fn setup_audio_receiver(
        &self,
        session_id: &str,
        audio_sender: mpsc::UnboundedSender<AudioSample>,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.audio_receiver = Some(audio_sender);
            Ok(())
        } else {
            Err(Error::WebRTC(format!("Session {} not found", session_id)))
        }
    }

    /// Create a new screen sharing session
    pub async fn create_session(&self, session_id: String) -> Result<String> {
        let config = RTCConfiguration {
            ice_servers: vec![],
            ..Default::default()
        };

        let peer_connection = Arc::new(
            self.api
                .new_peer_connection(config)
                .await
                .map_err(|e| Error::WebRTC(format!("Failed to create peer connection: {}", e)))?,
        );

        let session = ScreenSharingSession {
            peer_connection: peer_connection.clone(),
            session_id: session_id.clone(),
            is_active: false,
            audio_receiver: None,
        };

        // Add data channel for control messages
        let data_channel = peer_connection
            .create_data_channel("control", None)
            .await
            .map_err(|e| Error::WebRTC(format!("Failed to create data channel: {}", e)))?;

        // Set up data channel handlers
        let session_id_clone1 = session_id.clone();
        let sessions_clone = self.sessions.clone();
        data_channel.on_open(Box::new(move || {
            let session_id = session_id_clone1.clone();
            tracing::info!(
                "Screen sharing data channel opened for session {}",
                session_id
            );
            Box::pin(async move {
                let mut sessions = sessions_clone.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.is_active = true;
                }
            })
        }));

        let session_id_clone2 = session_id.clone();
        data_channel.on_close(Box::new(move || {
            tracing::info!(
                "Screen sharing data channel closed for session {}",
                session_id_clone2
            );
            Box::pin(async move {})
        }));

        // Set up audio track handling for voice input
        let session_id_clone3 = session_id.clone();
        peer_connection.on_track(Box::new(move |track: Arc<TrackRemote>, _receiver, _| {
            let session_id = session_id_clone3.clone();
            Box::pin(async move {
                tracing::info!(
                    "Received audio track for session {} - voice recognition integration pending",
                    session_id
                );
                // TODO: Implement RTP packet reading and voice recognition
                // This requires proper codec handling and integration with the voice processing pipeline
            })
        }));

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        // Create offer
        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| Error::WebRTC(format!("Failed to create offer: {}", e)))?;

        // Set local description
        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| Error::WebRTC(format!("Failed to set local description: {}", e)))?;

        // Return offer as JSON string
        let offer_json = serde_json::json!({
            "type": "offer",
            "sdp": offer.sdp
        });

        Ok(serde_json::to_string(&offer_json)
            .map_err(|e| Error::Serialization(format!("Failed to serialize offer: {}", e)))?)
    }

    /// Handle answer from remote peer
    pub async fn handle_answer(&self, session_id: &str, answer_sdp: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| Error::WebRTC(format!("Session {} not found", session_id)))?;

        let answer = RTCSessionDescription::answer(answer_sdp.to_string())
            .map_err(|e| Error::WebRTC(format!("Invalid answer SDP: {}", e)))?;

        session
            .peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| Error::WebRTC(format!("Failed to set remote description: {}", e)))?;

        tracing::info!("Screen sharing session {} established", session_id);
        Ok(())
    }

    /// End a screen sharing session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(session_id) {
            session
                .peer_connection
                .close()
                .await
                .map_err(|e| Error::WebRTC(format!("Failed to close peer connection: {}", e)))?;
            tracing::info!("Screen sharing session {} ended", session_id);
        }
        Ok(())
    }

    /// Get active sessions
    pub async fn get_active_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|(_, session)| session.is_active)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Process voice audio from remote device
    pub async fn process_voice_audio(
        &self,
        session_id: &str,
        audio: AudioSample,
    ) -> Result<Option<String>> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            if let Some(audio_sender) = &session.audio_receiver {
                // Send audio to voice recognition pipeline
                audio_sender.send(audio).map_err(|e| {
                    Error::WebRTC(format!("Failed to send audio for processing: {}", e))
                })?;

                // For now, return placeholder - actual voice recognition would be handled asynchronously
                Ok(Some("Voice command processed".to_string()))
            } else {
                Err(Error::WebRTC(format!(
                    "No audio receiver configured for session {}",
                    session_id
                )))
            }
        } else {
            Err(Error::WebRTC(format!("Session {} not found", session_id)))
        }
    }
}

impl Default for ScreenSharingManager {
    fn default() -> Self {
        Self::new()
    }
}
