use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Représente le type de nœud dans l'arbre B-TREE NHTML.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum NodeType {
    Element,
    Text,
    Root,
}

/// Structure fondamentale d'un nœud.
/// Chaque nœud est versionné pour permettre une resynchronisation différentielle fiable.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub node_type: NodeType,
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<u32>,
    pub version: u32, // Incrémenté à chaque modification (Patch)
}

/// Opération de modification unitaire (PatchOp).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchOp {
    pub op_code: u8, // Code binaire de l'opération (SET_TEXT=0x01, etc.)
    pub node_id: u32,
    pub value: String,
    pub attr_name: Option<String>,
    pub version: u32, // La version cible du nœud après application
}

/// Entrée dans le journal des événements pour le Record/Replay.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: String, // "CLICK", "INPUT", "SCROLL", etc.
    pub node_id: u32,
    pub payload: String,
}

/// État complet d'une session utilisateur.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub nodes: HashMap<u32, Node>, // Table de hachage de l'arbre actuel
    pub root_id: u32,
    pub last_event_id: u64,
}

impl SessionState {
    pub fn new(id: String) -> Self {
        Self {
            session_id: id,
            nodes: HashMap::new(),
            root_id: 0,
            last_event_id: 0,
        }
    }
    
    /// Incrémente la version d'un nœud après une modification.
    pub fn touch_node(&mut self, node_id: u32) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.version += 1;
        }
    }
}
