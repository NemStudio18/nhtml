/// compiler/mod.rs
/// Lit un fichier .nhtml, extrait les attributs n-,
/// produit le B-TREE binaire + la HandlerTable.

pub mod btree_builder;
pub mod handler_table;

use scraper::{Html, ElementRef, Selector, Node};
use std::collections::HashMap;
use tracing::warn;
use crate::proto::{LocalActionEntry,
    LA_ADD_CLASS, LA_REMOVE_CLASS, LA_TOGGLE_CLASS, LA_SET_STYLE,
    LA_CSS_VAR_SCROLL, LA_CSS_VAR_MOUSE_X, LA_CSS_VAR_MOUSE_Y, LA_CSS_VAR_MOUSE_PX,
    LA_TOGGLE_TARGET, LA_DRAG_ENABLE,
    LA_TRIG_HOVER, LA_TRIG_SCROLL_VP, LA_TRIG_SCROLL_PROG,
    LA_TRIG_MOUSEMOVE_WIN, LA_TRIG_MOUSEMOVE_SELF,
    LA_TRIG_CLICK_LOCAL, LA_TRIG_DRAG,
    LA_FLAG_ONCE, LA_FLAG_REVERSE_LEAVE, LA_FLAG_SCOPE_SELF,
};

// ─── Structures de sortie ──────────────────────────────────────────────────

/// Métadonnées d'un nœud issues des attributs n-
#[derive(Debug, Clone, Default)]
pub struct NAttrs {
    pub n_id           : Option<String>,
    pub n_click        : Option<String>,
    pub n_submit       : Option<String>,
    pub n_input        : Option<String>,
    pub n_change       : Option<String>,
    pub n_keydown      : Option<String>,
    pub n_focus        : Option<String>,
    pub n_blur         : Option<String>,
    pub n_model        : Option<String>,
    pub n_text         : Option<String>,
    pub n_live         : bool,
    pub n_prevent      : bool,
    pub n_once         : bool,
    pub n_debounce_ms  : u16,

    // ── Local Actions (v0.2.1) ──────────────────────────────────────────
    // Hover
    pub n_hover_add    : Option<String>,  // n-hover-add="classe"
    pub n_hover_remove : Option<String>,  // n-hover-remove="classe"
    pub n_hover_toggle : Option<String>,  // n-hover-toggle="classe"
    pub n_hover_style  : Option<String>,  // n-hover-style="prop:val"
    // Scroll
    pub n_scroll_var   : Option<String>,  // n-scroll-var="--nom"
    pub n_scroll_add   : Option<String>,  // n-scroll-add="classe"
    pub n_scroll_remove: Option<String>,  // n-scroll-remove="classe"
    pub n_scroll_threshold: f32,          // n-scroll-threshold="0.2"
    pub n_scroll_once  : bool,            // n-scroll-once
    // Mouse
    pub n_mouse_var_x  : Option<String>,  // n-mouse-var-x="--rx"
    pub n_mouse_var_y  : Option<String>,  // n-mouse-var-y="--ry"
    pub n_mouse_px     : Option<String>,  // n-mouse-px="--cx" (pixels absolus)
    pub n_mouse_scope_self: bool,         // n-mouse-scope="self"
    // Toggle
    pub n_toggle_target: Option<String>,  // n-toggle-target="nid"
    pub n_toggle_class : Option<String>,  // n-toggle-class="open"
    pub n_toggle_outside: bool,           // n-toggle-close-on-outside
    // Drag
    pub n_draggable    : bool,            // n-draggable
    pub n_drag_add     : Option<String>,  // n-drag-add="classe"
    pub n_drop_target  : Option<String>,  // n-drop-target="groupe"
    // Click local
    pub n_click_local  : Option<String>,  // n-click-local="target-nid"
}

impl NAttrs {
    /// Retourne le premier handler d'événement trouvé
    pub fn primary_handler(&self) -> Option<&str> {
        self.n_click.as_deref()
            .or(self.n_submit.as_deref())
            .or(self.n_input.as_deref())
            .or(self.n_change.as_deref())
            .or(self.n_focus.as_deref())
            .or(self.n_blur.as_deref())
    }

    /// Calcule le listen_mask (bits d'événements)
    pub fn listen_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.n_click.is_some()   { mask |= 0x01; }
        if self.n_input.is_some()   { mask |= 0x02; }
        if self.n_submit.is_some()  { mask |= 0x04; }
        if self.n_keydown.is_some() { mask |= 0x08; }
        if self.n_focus.is_some() || self.n_blur.is_some() { mask |= 0x10; }
        mask
    }

    /// Calcule le behavior_flags
    pub fn behavior_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.n_live    { flags |= 0x01; }
        if self.n_prevent { flags |= 0x02; }
        if self.n_once    { flags |= 0x04; }
        flags
    }

    /// Debounce en unités de 100ms (arrondi supérieur)
    pub fn debounce_100ms(&self) -> u8 {
        ((self.n_debounce_ms + 99) / 100).min(255) as u8
    }
}

/// Représentation d'un nœud après parsing
#[derive(Debug, Clone)]
pub struct NodeSpec {
    pub id             : u16,
    #[allow(dead_code)]
    pub parent_id      : u16,
    #[allow(dead_code)]
    pub node_type      : u8,          // 0x01=element, 0x02=text
    #[allow(dead_code)]
    pub tag            : String,
    #[allow(dead_code)]
    pub attrs          : Vec<(String, String)>,  // attrs HTML standards uniquement
    pub n_attrs        : NAttrs,
    pub text           : String,
    pub children       : Vec<NodeSpec>,
}

impl NodeSpec {
    pub fn to_html(&self) -> String {
        if self.id == 0 && self.tag == "body" {
            let mut html = String::new();
            for child in &self.children {
                html.push_str(&child.to_html());
            }
            return html;
        }

        let mut html = format!("<{}", self.tag);
        
        // Injecter n-id si nécessaire
        let needs_nid = self.n_attrs.n_id.is_some() || self.n_attrs.primary_handler().is_some() || self.n_attrs.n_model.is_some();
        if needs_nid {
            let nid = self.n_attrs.n_id.clone().unwrap_or_else(|| format!("_gen_{}", self.id));
            html.push_str(&format!(" n-id=\"{}\"", html_escape::encode_double_quoted_attribute(&nid)));
        }

        for (k, v) in &self.attrs {
            html.push_str(&format!(" {}=\"{}\"", k, html_escape::encode_double_quoted_attribute(v)));
        }
        html.push('>');

        if !self.text.is_empty() {
            html.push_str(&html_escape::encode_safe(&self.text));
        }

        for child in &self.children {
            html.push_str(&child.to_html());
        }

        html.push_str(&format!("</{}>", self.tag));
        html
    }
}


/// Table de correspondance n-id métier → node_id binaire
pub type NidMap = HashMap<String, u16>;

/// Résultat de la compilation d'un .nhtml
#[derive(Clone)]
pub struct CompileResult {
    pub root         : NodeSpec,
    #[allow(dead_code)]
    pub nid_map      : NidMap,     // "compteur" → 2
    pub states       : Vec<(u16, u32, String, String)>,
    pub btree_bytes  : Vec<u8>,    // payload B-TREE (avant wrap PKT_BTREE)
    pub bind_packets : Vec<Vec<u8>>, // paquets 0x04 BIND prêts à envoyer
    pub html         : String,     // HTML avec n-id injectés
}

// ─── Attributs n- à exclure du B-TREE ─────────────────────────────────────

const N_ATTRS: &[&str] = &[
    "n-id", "n-click", "n-submit", "n-input", "n-change",
    "n-keydown", "n-focus", "n-blur", "n-model", "n-text",
    "n-live", "n-prevent", "n-once", "n-debounce", "n-if",
    "n-class", "n-confirm",
    // Local Actions v0.2.1
    "n-hover-add", "n-hover-remove", "n-hover-toggle", "n-hover-style",
    "n-scroll-var", "n-scroll-add", "n-scroll-remove",
    "n-scroll-threshold", "n-scroll-once",
    "n-mouse-var-x", "n-mouse-var-y", "n-mouse-px", "n-mouse-scope",
    "n-toggle-target", "n-toggle-class", "n-toggle-close-on-outside",
    "n-draggable", "n-drag-add", "n-drop-target",
    "n-click-local",
];

fn is_n_attr(name: &str) -> bool {
    N_ATTRS.contains(&name)
}

// ─── Compilateur principal ─────────────────────────────────────────────────

pub struct NhtmlCompiler {
    next_id  : u16,
    nid_map  : NidMap,
}

impl NhtmlCompiler {
    pub fn new() -> Self {
        Self { next_id: 1, nid_map: HashMap::new() }
    }

    fn alloc_id(&mut self) -> u16 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Point d'entrée : compile un fichier .nhtml complet
    pub fn compile(source: &str) -> CompileResult {
        let mut compiler = NhtmlCompiler::new();
        let document     = Html::parse_document(source);

        // Extraire le head
        let head_html = if let Ok(head_sel) = Selector::parse("head") {
            if let Some(head) = document.select(&head_sel).next() {
                head.inner_html()
            } else {
                "".to_string()
            }
        } else { "".to_string() };

        // Trouver le body
        let body_ref = if let Ok(body_sel) = Selector::parse("body") {
            document.select(&body_sel).next()
        } else { None };

        let mut root_nodes = Vec::new();

        if let Some(body) = body_ref {
            for child in body.children() {
                if let Node::Element(_) = child.value() {
                    if let Some(child_ref) = ElementRef::wrap(child) {
                        root_nodes.push(compiler.parse_element(child_ref, 0));
                    }
                }
            }
        } else {
            // Fallback : prendre le premier élément du document
            if let Ok(first_sel) = Selector::parse(":root > *") {
                if let Some(first) = document.select(&first_sel).next() {
                    root_nodes.push(compiler.parse_element(first, 0));
                }
            }
        }

        if root_nodes.is_empty() {
            warn!("Aucun élément racine trouvé dans le .nhtml");
            // On renvoie un résultat vide mais valide au lieu de paniquer
            return CompileResult {
                root: NodeSpec { id: 0, parent_id: 0, node_type: 0x01, tag: "div".to_string(), attrs: Vec::new(), n_attrs: NAttrs::default(), text: "No content found".to_string(), children: Vec::new() },
                nid_map: HashMap::new(),
                states: Vec::new(),
                btree_bytes: Vec::new(),
                bind_packets: Vec::new(),
                html: "<!-- Empty Document -->".to_string(),
            };
        }

        // --- NOUVEAU : Format A (Liste d'états) attendu par bridge.js ---
        let mut states = Vec::new();
        for node in &root_nodes {
            Self::collect_states(node, &mut states);
        }
        let btree_bytes = crate::proto::serialize_nodes(&states);

        // Pour que build_from_tree (dans Session::new) ramasse TOUS les handlers,
        // on crée un nœud virtuel "body" qui contient tous les root_nodes.
        let virtual_root = NodeSpec {
            id: 0,
            parent_id: 0,
            node_type: 0x01,
            tag: "body".to_string(),
            attrs: Vec::new(),
            text: String::new(),
            n_attrs: crate::compiler::NAttrs::default(),
            children: root_nodes,
        };

        // Construire les paquets BIND
        let bind_packets = compiler.build_bind_packets(&virtual_root);

        CompileResult {
            btree_bytes,
            bind_packets,
            nid_map: compiler.nid_map,
            root: virtual_root.clone(),
            states,
            html: format!("<!DOCTYPE html><html><head>{}</head><body>{}</body></html>", head_html, virtual_root.to_html()),
        }
    }

    fn collect_states(node: &NodeSpec, states: &mut Vec<(u16, u32, String, String)>) {
        if let Some(ref nid) = node.n_attrs.n_id {
            states.push((node.id, 0, nid.clone(), node.text.clone()));
        }
        for child in &node.children {
            Self::collect_states(child, states);
        }
    }

    /// Parse récursivement un élément HTML
    fn parse_element(&mut self, el: ElementRef, parent_id: u16) -> NodeSpec {
        let id  = self.alloc_id();
        let tag = el.value().name().to_lowercase();

        // Séparer attributs HTML standards et attributs n-
        let mut html_attrs = Vec::new();
        let mut n_attrs    = NAttrs::default();

        for attr in el.value().attrs() {
            let name = attr.0;
            let val  = attr.1;

            if is_n_attr(name) {
                self.parse_n_attr(&mut n_attrs, name, val);
            } else {
                html_attrs.push((name.to_string(), val.to_string()));
            }
        }

        // Enregistrer le n-id dans la map
        if let Some(ref nid) = n_attrs.n_id {
            self.nid_map.insert(nid.clone(), id);
        }

        // Parser les enfants
        let mut children = Vec::new();
        let mut text     = String::new();

        for child in el.children() {
            match child.value() {
                Node::Element(_) => {
                    if let Some(child_ref) = ElementRef::wrap(child) {
                        children.push(self.parse_element(child_ref, id));
                    }
                }
                Node::Text(t) => {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        text = trimmed.to_string();
                    }
                }
                _ => {}
            }
        }

        NodeSpec {
            id,
            parent_id,
            node_type: 0x01,
            tag,
            attrs: html_attrs,
            text,
            n_attrs,
            children,
        }
    }

    fn parse_n_attr(&self, n: &mut NAttrs, name: &str, val: &str) {
        match name {
            // ── Attributs v0.2 ───────────────────────────────────────────────────────
            "n-id"       => n.n_id       = Some(val.to_string()),
            "n-click"    => n.n_click    = Some(val.to_string()),
            "n-submit"   => n.n_submit   = Some(val.to_string()),
            "n-input"    => n.n_input    = Some(val.to_string()),
            "n-change"   => n.n_change   = Some(val.to_string()),
            "n-keydown"  => n.n_keydown  = Some(val.to_string()),
            "n-focus"    => n.n_focus    = Some(val.to_string()),
            "n-blur"     => n.n_blur     = Some(val.to_string()),
            "n-model"    => n.n_model    = Some(val.to_string()),
            "n-text"     => n.n_text     = Some(val.to_string()),
            "n-live"     => n.n_live     = true,
            "n-prevent"  => n.n_prevent  = true,
            "n-once"     => n.n_once     = true,
            "n-debounce" => {
                n.n_debounce_ms = val.parse().unwrap_or_else(|_| {
                    warn!("n-debounce invalide: {}", val);
                    0
                });
            }
            // ── Local Actions v0.2.1 ───────────────────────────────────────────
            "n-hover-add"    => n.n_hover_add    = Some(val.to_string()),
            "n-hover-remove" => n.n_hover_remove = Some(val.to_string()),
            "n-hover-toggle" => n.n_hover_toggle = Some(val.to_string()),
            "n-hover-style"  => n.n_hover_style  = Some(val.to_string()),
            "n-scroll-var"   => n.n_scroll_var   = Some(val.to_string()),
            "n-scroll-add"   => n.n_scroll_add   = Some(val.to_string()),
            "n-scroll-remove"=> n.n_scroll_remove= Some(val.to_string()),
            "n-scroll-threshold" => {
                n.n_scroll_threshold = val.parse().unwrap_or(0.1);
            }
            "n-scroll-once"  => n.n_scroll_once  = true,
            "n-mouse-var-x" => n.n_mouse_var_x = Some(val.to_string()),
            "n-mouse-var-y" => n.n_mouse_var_y = Some(val.to_string()),
            "n-mouse-px"    => n.n_mouse_px    = Some(val.to_string()),
            "n-mouse-scope" => n.n_mouse_scope_self = val == "self",
            "n-toggle-target"            => n.n_toggle_target  = Some(val.to_string()),
            "n-toggle-class"             => n.n_toggle_class   = Some(val.to_string()),
            "n-toggle-close-on-outside"  => n.n_toggle_outside = true,
            "n-draggable" => n.n_draggable  = true,
            "n-drag-add"  => n.n_drag_add   = Some(val.to_string()),
            "n-drop-target" => n.n_drop_target = Some(val.to_string()),
            "n-click-local" => n.n_click_local = Some(val.to_string()),
            _ => {}
        }
    }

    /// Génère les paquets BIND pour tous les nœuds ayant des attributs n-
    fn build_bind_packets(&self, node: &NodeSpec) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        self.collect_binds(node, &mut packets);
        packets
    }

    fn collect_binds(&self, node: &NodeSpec, packets: &mut Vec<Vec<u8>>) {
        let n = &node.n_attrs;

        // Un BIND est nécessaire si le nœud a au moins un attribut n-
        let needs_bind = n.n_id.is_some()
            || n.primary_handler().is_some()
            || n.n_model.is_some()
            || n.n_text.is_some()
            || n.n_live
            || n.listen_mask() != 0;

        if needs_bind {
            let selector = if let Some(ref nid) = n.n_id {
                format!("[n-id={}]", nid)
            } else {
                format!("[n-id=\"_gen_{}\"]", node.id)
            };

            // ── Construire les Local Actions ───────────────────────────────────
            let mut local_actions: Vec<crate::proto::LocalActionEntry> = Vec::new();

            // Hover
            if let Some(ref cls) = n.n_hover_add {
                local_actions.push(LocalActionEntry {
                    action_type: LA_ADD_CLASS, trigger_type: LA_TRIG_HOVER,
                    param: cls.clone(),
                    flags: LA_FLAG_REVERSE_LEAVE,
                    threshold_x10: 0,
                });
            }
            if let Some(ref cls) = n.n_hover_remove {
                local_actions.push(LocalActionEntry {
                    action_type: LA_REMOVE_CLASS, trigger_type: LA_TRIG_HOVER,
                    param: cls.clone(), flags: LA_FLAG_REVERSE_LEAVE, threshold_x10: 0,
                });
            }
            if let Some(ref cls) = n.n_hover_toggle {
                local_actions.push(LocalActionEntry {
                    action_type: LA_TOGGLE_CLASS, trigger_type: LA_TRIG_HOVER,
                    param: cls.clone(), flags: 0, threshold_x10: 0,
                });
            }
            if let Some(ref pv) = n.n_hover_style {
                local_actions.push(LocalActionEntry {
                    action_type: LA_SET_STYLE, trigger_type: LA_TRIG_HOVER,
                    param: pv.clone(), flags: LA_FLAG_REVERSE_LEAVE, threshold_x10: 0,
                });
            }

            // Scroll viewport
            if let Some(ref cls) = n.n_scroll_add {
                let thr = (n.n_scroll_threshold * 10.0).round() as u8;
                let flags = if n.n_scroll_once { LA_FLAG_ONCE } else { 0 };
                local_actions.push(LocalActionEntry {
                    action_type: LA_ADD_CLASS, trigger_type: LA_TRIG_SCROLL_VP,
                    param: cls.clone(), flags, threshold_x10: thr,
                });
            }
            if let Some(ref cls) = n.n_scroll_remove {
                let thr = (n.n_scroll_threshold * 10.0).round() as u8;
                local_actions.push(LocalActionEntry {
                    action_type: LA_REMOVE_CLASS, trigger_type: LA_TRIG_SCROLL_VP,
                    param: cls.clone(), flags: 0, threshold_x10: thr,
                });
            }

            // Scroll progress (CSS var)
            if let Some(ref var) = n.n_scroll_var {
                local_actions.push(LocalActionEntry {
                    action_type: LA_CSS_VAR_SCROLL, trigger_type: LA_TRIG_SCROLL_PROG,
                    param: var.clone(), flags: 0, threshold_x10: 0,
                });
            }

            // Mouse vars
            if let Some(ref var) = n.n_mouse_var_x {
                let trig = if n.n_mouse_scope_self { LA_TRIG_MOUSEMOVE_SELF } else { LA_TRIG_MOUSEMOVE_WIN };
                local_actions.push(LocalActionEntry {
                    action_type: LA_CSS_VAR_MOUSE_X, trigger_type: trig,
                    param: var.clone(), flags: if n.n_mouse_scope_self { LA_FLAG_SCOPE_SELF } else { 0 },
                    threshold_x10: 0,
                });
            }
            if let Some(ref var) = n.n_mouse_var_y {
                let trig = if n.n_mouse_scope_self { LA_TRIG_MOUSEMOVE_SELF } else { LA_TRIG_MOUSEMOVE_WIN };
                local_actions.push(LocalActionEntry {
                    action_type: LA_CSS_VAR_MOUSE_Y, trigger_type: trig,
                    param: var.clone(), flags: if n.n_mouse_scope_self { LA_FLAG_SCOPE_SELF } else { 0 },
                    threshold_x10: 0,
                });
            }
            if let Some(ref var) = n.n_mouse_px {
                local_actions.push(LocalActionEntry {
                    action_type: LA_CSS_VAR_MOUSE_PX, trigger_type: LA_TRIG_MOUSEMOVE_WIN,
                    param: var.clone(), flags: 0, threshold_x10: 0,
                });
            }

            // Toggle local
            if let Some(ref target_nid) = n.n_toggle_target {
                let cls = n.n_toggle_class.as_deref().unwrap_or("open");
                local_actions.push(LocalActionEntry {
                    action_type: LA_TOGGLE_TARGET, trigger_type: LA_TRIG_CLICK_LOCAL,
                    param: format!("{}:{}", target_nid, cls), flags: 0, threshold_x10: 0,
                });
            }

            // Drag
            if n.n_draggable {
                let group = n.n_drop_target.as_deref().unwrap_or("");
                local_actions.push(LocalActionEntry {
                    action_type: LA_DRAG_ENABLE, trigger_type: LA_TRIG_DRAG,
                    param: group.to_string(), flags: 0, threshold_x10: 0,
                });
            }
            if let Some(ref cls) = n.n_drag_add {
                local_actions.push(LocalActionEntry {
                    action_type: LA_ADD_CLASS, trigger_type: LA_TRIG_DRAG,
                    param: cls.clone(), flags: LA_FLAG_REVERSE_LEAVE, threshold_x10: 0,
                });
            }

            let nid_str = n.n_id.clone().unwrap_or_else(|| format!("_gen_{}", node.id));
            let packet = crate::proto::bind(crate::proto::BindParams {
                node_id        : node.id,
                nid            : &nid_str,
                selector       : &selector,
                listen_mask    : n.listen_mask(),
                behavior_flags : n.behavior_flags(),
                debounce_100ms : n.debounce_100ms(),
                handler        : n.primary_handler().unwrap_or(""),
                n_model        : n.n_model.as_deref().unwrap_or(""),
                n_text         : n.n_text.as_deref().unwrap_or(""),
                local_actions,
            });

            packets.push(packet);
        }

        for child in &node.children {
            self.collect_binds(child, packets);
        }
    }
}
