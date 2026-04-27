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

/// Erreurs personnalisées pour le Gateway NHTML
#[derive(Debug)]
pub enum GatewayError {
    PhpNotFound(String),
    PhpExecutionError(String),
    FastCgiError(String),
    SocketError(String),
    DatabaseError(String),
    ConfigError(String),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PhpNotFound(s) => write!(f, "Exécutable PHP non trouvé : {}", s),
            Self::PhpExecutionError(s) => write!(f, "Erreur lors de l'exécution du script PHP : {}", s),
            Self::FastCgiError(s) => write!(f, "Erreur FastCGI / PHP-FPM : {}", s),
            Self::SocketError(s) => write!(f, "Erreur de communication WebSocket : {}", s),
            Self::DatabaseError(s) => write!(f, "Erreur de base de données : {}", s),
            Self::ConfigError(s) => write!(f, "Erreur de configuration : {}", s),
        }
    }
}

impl std::error::Error for GatewayError {}
pub type Result<T> = std::result::Result<T, GatewayError>;
