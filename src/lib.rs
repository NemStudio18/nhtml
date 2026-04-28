pub mod core;
pub mod cli;
pub mod supervisor;
pub mod watcher;
pub mod session;
pub mod proto;
pub mod decoder;
pub mod config;
pub mod compiler;
pub mod socket;
pub mod cluster;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringEvent {
    pub timestamp: String,
    pub direction: String, // "IN", "OUT", "SYS"
    pub pkt_type: u8,
    pub size: usize,
    pub session_id: String,
    pub node_id: Option<u16>,
    pub latency_ms: Option<u64>,
    pub compression_ratio: Option<f32>,
    pub handler: Option<String>,
    pub details: Option<String>,
}
