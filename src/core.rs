use serde::{Serialize, Deserialize};

/// État minimal d'une session utilisateur.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub last_event_id: u64,
    pub last_version: u32,
}

impl SessionState {
    pub fn new(id: String) -> Self {
        Self {
            session_id: id,
            last_event_id: 0,
            last_version: 0,
        }
    }
}
