/// compiler/handler_table.rs
/// Table des liaisons n-id → node_id → handler.
/// Sérialisée en JSON et transmise au PHP via stdin pour le dispatch.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Une entrée de la table : tout ce dont le PHP a besoin pour dispatcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerEntry {
    pub node_id   : u16,
    pub n_id      : Option<String>,
    pub handler   : Option<String>,  // "module.action" ou "module.action:param"
    pub n_model   : Option<String>,
    pub n_text    : Option<String>,
    pub n_live    : bool,
    pub listen    : Vec<String>,     // ["click", "input", ...]
}

/// La table complète pour une page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandlerTable {
    /// node_id → HandlerEntry
    pub by_id  : HashMap<u16, HandlerEntry>,
    /// n-id métier → node_id (pour ciblage PHP via NhtmlPatch::setText("compteur", ...))
    pub nid_map: HashMap<String, u16>,
}

impl HandlerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: HandlerEntry) {
        if let Some(ref nid) = entry.n_id {
            self.nid_map.insert(nid.clone(), entry.node_id);
        }
        self.by_id.insert(entry.node_id, entry);
    }

    /// Résoudre un n-id métier en node_id binaire
    pub fn resolve_nid(&self, nid: &str) -> Option<u16> {
        self.nid_map.get(nid).copied()
    }

    /// Sérialiser en JSON pour transmission au PHP
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Construire la HandlerTable depuis le NodeSpec tree
pub fn build_from_tree(root: &super::NodeSpec) -> HandlerTable {
    let mut table = HandlerTable::new();
    collect_entries(root, &mut table);
    table
}

fn collect_entries(node: &super::NodeSpec, table: &mut HandlerTable) {
    let n = &node.n_attrs;

    let mut listen = Vec::new();
    if n.n_click.is_some()   { listen.push("click".to_string()); }
    if n.n_input.is_some()   { listen.push("input".to_string()); }
    if n.n_submit.is_some()  { listen.push("submit".to_string()); }
    if n.n_keydown.is_some() { listen.push("keydown".to_string()); }
    if n.n_focus.is_some()   { listen.push("focus".to_string()); }
    if n.n_blur.is_some()    { listen.push("blur".to_string()); }

    let has_n_attrs = n.n_id.is_some()
        || n.primary_handler().is_some()
        || n.n_model.is_some()
        || n.n_text.is_some()
        || n.n_live
        || !listen.is_empty();

    if has_n_attrs {
        table.insert(HandlerEntry {
            node_id : node.id,
            n_id    : n.n_id.clone(),
            handler : n.primary_handler().map(str::to_string),
            n_model : n.n_model.clone(),
            n_text  : n.n_text.clone(),
            n_live  : n.n_live,
            listen,
        });
    }

    for child in &node.children {
        collect_entries(child, table);
    }
}
