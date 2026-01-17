use crate::domain::entities::{RecognitionSession, SessionStatus};
use crate::infrastructure::repositories::SessionRepository;
use crate::shared::{Result, Error, SessionId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory session repository for active sessions
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<SessionId, RecognitionSession>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: &RecognitionSession) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        tracing::debug!("Saved session: {}", session.id);
        Ok(())
    }

    async fn find_by_id(&self, id: &SessionId) -> Result<Option<RecognitionSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(id).cloned())
    }

    async fn find_active_sessions(&self) -> Result<Vec<RecognitionSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .cloned()
            .collect())
    }
}

impl InMemorySessionRepository {
    /// Clean up completed or failed sessions older than the given duration
    pub fn cleanup_old_sessions(&self, max_age: std::time::Duration) {
        let mut sessions = self.sessions.write().unwrap();
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::hours(1));

        sessions.retain(|_, session| {
            if session.status == SessionStatus::Active {
                return true;
            }
            session.start_time > cutoff
        });
    }

    /// Get session count
    pub fn count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.len()
    }

    /// Get active session count
    pub fn active_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.values().filter(|s| s.status == SessionStatus::Active).count()
    }
}
