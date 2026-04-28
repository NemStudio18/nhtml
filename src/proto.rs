/// proto.rs — Binary serialization for Nhtml v0.7.0
/// Rust source of truth for NBPS (Nhtml Binary Packet Specification).
/// Must remain in sync with SPEC.md and bridge.js.

// ─── Constantes ────────────────────────────────────────────────────────────

pub const PKT_HELLO  : u8 = 0x01;
pub const PKT_EVENT  : u8 = 0x02;
pub const PKT_PATCH  : u8 = 0x03;
pub const PKT_BIND   : u8 = 0x04;
pub const PKT_SYNC   : u8 = 0x05;
pub const PKT_BTREE  : u8 = 0x07;
pub const PKT_PUSH_PATCH: u8 = 0x08; // Client -> Server (Zero-Server mode)
pub const PKT_PING   : u8 = 0x09;
pub const PKT_LOG    : u8 = 0x10;
pub const PKT_ERR    : u8 = 0x7F;

pub const SCOPE_OTHERS : u8 = 0x01;
pub const SCOPE_ALL    : u8 = 0x02;
pub const SCOPE_ROOM   : u8 = 0x03;
pub const SCOPE_DIRECT : u8 = 0x04;

pub const OP_SET_TEXT      : u8 = 0x01;
pub const OP_SET_ATTR      : u8 = 0x02;
pub const OP_DEL_ATTR      : u8 = 0x03;
pub const OP_ADD_CLASS     : u8 = 0x04;
pub const OP_DEL_CLASS     : u8 = 0x05;
pub const OP_INSERT_BEFORE : u8 = 0x06;
pub const OP_INSERT_AFTER  : u8 = 0x07;
pub const OP_REMOVE        : u8 = 0x08;
pub const OP_SET_STYLE     : u8 = 0x09;
pub const OP_REPLACE_INNER : u8 = 0x0A;
pub const OP_APPEND_HTML   : u8 = 0x0B;
pub const OP_SCROLL_TO     : u8 = 0x0C;
pub const OP_FOCUS         : u8 = 0x0D;

pub const LISTEN_CLICK   : u8 = 0x01;
pub const LISTEN_INPUT   : u8 = 0x02;
pub const LISTEN_SUBMIT  : u8 = 0x04;
pub const LISTEN_KEYDOWN : u8 = 0x08;
pub const LISTEN_SCROLL  : u8 = 0x10;

pub const FLAG_N_LIVE    : u8 = 0x01;
pub const FLAG_N_PREVENT : u8 = 0x02;
pub const FLAG_N_ONCE    : u8 = 0x04;

pub const SEV_WARN  : u8 = 0x01;
pub const SEV_ERROR : u8 = 0x02;
pub const SEV_FATAL : u8 = 0x03;

// ─── Local Actions — types d'effets (spec v0.2.1, table §5) ─────────────────

pub const LA_ADD_CLASS         : u8 = 0x01;
pub const LA_REMOVE_CLASS      : u8 = 0x02;
pub const LA_TOGGLE_CLASS      : u8 = 0x03;
pub const LA_SET_STYLE         : u8 = 0x04;
pub const LA_CSS_VAR_SCROLL    : u8 = 0x05;
pub const LA_CSS_VAR_MOUSE_X   : u8 = 0x06;
pub const LA_CSS_VAR_MOUSE_Y   : u8 = 0x07;
pub const LA_CSS_VAR_MOUSE_PX  : u8 = 0x08;
pub const LA_TOGGLE_TARGET     : u8 = 0x09;
pub const LA_DRAG_ENABLE       : u8 = 0x0A;

// ─── Local Actions — déclencheurs (spec v0.2.1, table §6) ──────────────────

pub const LA_TRIG_HOVER         : u8 = 0x01;
pub const LA_TRIG_SCROLL_VP     : u8 = 0x02;
pub const LA_TRIG_SCROLL_PROG   : u8 = 0x03;
pub const LA_TRIG_MOUSEMOVE_WIN : u8 = 0x04;
pub const LA_TRIG_MOUSEMOVE_SELF: u8 = 0x05;
pub const LA_TRIG_FOCUS         : u8 = 0x06;
pub const LA_TRIG_CLICK_LOCAL   : u8 = 0x07;
pub const LA_TRIG_DRAG          : u8 = 0x08;

// Flags de comportement (bits)
pub const LA_FLAG_ONCE         : u8 = 0x01;
pub const LA_FLAG_REVERSE_LEAVE: u8 = 0x02;
pub const LA_FLAG_SCOPE_SELF   : u8 = 0x04;

// ─── Helpers d'écriture ────────────────────────────────────────────────────

#[inline]
fn push_u16(buf: &mut Vec<u8>, v: u16) {
    buf.push((v >> 8) as u8);
    buf.push((v & 0xFF) as u8);
}

#[inline]
fn push_u32(buf: &mut Vec<u8>, v: u32) {
    buf.push((v >> 24) as u8);
    buf.push((v >> 16) as u8);
    buf.push((v >> 8)  as u8);
    buf.push((v & 0xFF) as u8);
}

#[inline]
fn push_str8(buf: &mut Vec<u8>, s: &str) {
    buf.push(s.len() as u8);
    buf.extend_from_slice(s.as_bytes());
}

#[inline]
fn push_str16(buf: &mut Vec<u8>, s: &str) {
    push_u16(buf, s.len() as u16);
    buf.extend_from_slice(s.as_bytes());
}

fn wrap_packet(pkt_type: u8, payload: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(pkt_type);
    push_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(&payload);
    out
}

// ─── HELLO ─────────────────────────────────────────────────────────────────

pub fn hello(session_id: &str, secret: &[u8], last_seq: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(0x00); // Status: 0 (OK)
    push_str8(&mut payload, session_id);
    if secret.len() == 32 {
        payload.extend_from_slice(secret);
    } else {
        payload.extend_from_slice(&[0u8; 32]);
    }
    push_u32(&mut payload, last_seq);
    wrap_packet(PKT_HELLO, payload)
}

// ─── BIND ──────────────────────────────────────────────────────────────────

/// Une Local Action à inclure dans le paquet BIND (extension v0.2.1)
#[derive(Debug, Clone)]
pub struct LocalActionEntry {
    pub action_type    : u8,
    pub trigger_type   : u8,
    pub param          : String,  // classe, CSS var, "prop:val", n-id cible…
    pub flags          : u8,      // LA_FLAG_* bits
    pub threshold_x10  : u8,      // seuil scroll × 10 (15 = 0.15)
}

pub struct BindParams<'a> {
    pub node_id        : u16,
    pub nid           : &'a str,
    pub selector       : &'a str,
    pub listen_mask    : u8,
    pub behavior_flags : u8,
    pub debounce_100ms : u8,
    pub handler        : &'a str,
    pub n_model        : &'a str,
    pub n_text         : &'a str,
    pub local_actions  : Vec<LocalActionEntry>,  // extension v0.2.1
}

pub fn bind(p: BindParams) -> Vec<u8> {
    let mut payload = Vec::new();
    push_u16(&mut payload, p.node_id);
    push_str8(&mut payload, p.nid);
    push_str8(&mut payload, p.selector);
    payload.push(p.listen_mask);
    payload.push(p.behavior_flags);
    payload.push(p.debounce_100ms);
    push_str8(&mut payload, p.handler);
    push_str8(&mut payload, p.n_model);
    push_str8(&mut payload, p.n_text);

    // ── Extension v0.2.1 : Local Actions ─────────────────────────────────────
    payload.push(p.local_actions.len() as u8);
    for la in &p.local_actions {
        payload.push(la.action_type);
        payload.push(la.trigger_type);
        push_str8(&mut payload, &la.param);
        payload.push(la.flags);
        payload.push(la.threshold_x10);
    }

    wrap_packet(PKT_BIND, payload)
}

// ─── PATCH ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PatchOp {
    pub op_type   : u8,
    pub target_id : u16,
    pub version   : u32,
    pub data      : Vec<u8>,  // op_data pré-sérialisé (inclut la longueur interne si nécessaire)
}

impl PatchOp {
    pub fn set_text(target_id: u16, version: u32, text: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, text);
        Self { op_type: OP_SET_TEXT, target_id, version, data }
    }

    pub fn replace_inner(target_id: u16, version: u32, html: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, html);
        Self { op_type: OP_REPLACE_INNER, target_id, version, data }
    }

    pub fn append_html(target_id: u16, version: u32, html: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, html);
        Self { op_type: OP_APPEND_HTML, target_id, version, data }
    }

    pub fn insert_before(target_id: u16, version: u32, html: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, html);
        Self { op_type: OP_INSERT_BEFORE, target_id, version, data }
    }

    pub fn insert_after(target_id: u16, version: u32, html: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, html);
        Self { op_type: OP_INSERT_AFTER, target_id, version, data }
    }

    pub fn set_attr(target_id: u16, version: u32, key: &str, val: &str) -> Self {
        let mut data = Vec::new();
        push_str8(&mut data, key);
        push_str16(&mut data, val);
        Self { op_type: OP_SET_ATTR, target_id, version, data }
    }

    pub fn del_attr(target_id: u16, version: u32, key: &str) -> Self {
        let mut data = Vec::new();
        push_str8(&mut data, key);
        Self { op_type: OP_DEL_ATTR, target_id, version, data }
    }

    pub fn set_style(target_id: u16, version: u32, prop: &str, val: &str) -> Self {
        let mut data = Vec::new();
        push_str8(&mut data, prop);
        push_str16(&mut data, val);
        Self { op_type: OP_SET_STYLE, target_id, version, data }
    }

    pub fn scroll_to(target_id: u16, version: u32) -> Self {
        Self { op_type: OP_SCROLL_TO, target_id, version, data: Vec::new() }
    }

    pub fn add_class(target_id: u16, version: u32, class: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, class);
        Self { op_type: OP_ADD_CLASS, target_id, version, data }
    }

    pub fn del_class(target_id: u16, version: u32, class: &str) -> Self {
        let mut data = Vec::new();
        push_str16(&mut data, class);
        Self { op_type: OP_DEL_CLASS, target_id, version, data }
    }

    pub fn remove(target_id: u16, version: u32) -> Self {
        Self { op_type: OP_REMOVE, target_id, version, data: Vec::new() }
    }

    pub fn focus(target_id: u16, version: u32) -> Self {
        Self { op_type: OP_FOCUS, target_id, version, data: Vec::new() }
    }
}

pub fn patch(ops: &[PatchOp]) -> Vec<u8> {
    let mut payload = Vec::new();
    push_u16(&mut payload, ops.len() as u16); // OpCount (u16)
    for op in ops {
        push_u16(&mut payload, op.target_id); // TargetID (u16)
        payload.push(op.op_type);             // OpType (u8)
        push_u32(&mut payload, op.version);   // NodeVersion (u32)
        push_u16(&mut payload, op.data.len() as u16); // DataLen (u16) - AJOUTÉ
        payload.extend_from_slice(&op.data);
    }
    wrap_packet(PKT_PATCH, payload)
}

// ─── B-TREE ────────────────────────────────────────────────────────────────

pub fn serialize_nodes(nodes: &[(u16, u32, String, String)]) -> Vec<u8> {
    let mut buf = Vec::new();
    push_u16(&mut buf, nodes.len() as u16);
    for (id, ver, tag, val) in nodes {
        push_u16(&mut buf, *id);
        push_u32(&mut buf, *ver);
        push_str8(&mut buf, tag);
        push_str16(&mut buf, val);
    }
    buf
}

pub fn btree(nodes: &[(u16, u32, String, String)]) -> (Vec<u8>, f32) {
    let tree_payload = serialize_nodes(nodes);
    let (pkt, ratio) = wrap_btree(&tree_payload);
    (pkt, ratio)
}

pub fn wrap_btree(payload: &[u8]) -> (Vec<u8>, f32) {
    let orig_len = payload.len();
    let checksum = crc32fast::hash(payload);
    
    let mut comp_flag: u8 = 0x00;
    let mut final_payload = payload.to_vec();
    let mut ratio: f32 = 1.0;
    
    // Tentative de compression Zstd niveau 3
    if let Ok(compressed) = zstd::encode_all(payload, 3) {
        if compressed.len() < payload.len() {
            ratio = compressed.len() as f32 / orig_len.max(1) as f32;
            comp_flag = 0x01;
            final_payload = compressed;
        }
    }

    let mut header = Vec::new();
    header.push(comp_flag);
    push_u32(&mut header, orig_len as u32);
    push_u32(&mut header, checksum);

    let total_payload_len = header.len() + final_payload.len();
    let mut out = Vec::with_capacity(5 + total_payload_len);
    out.push(PKT_BTREE);
    push_u32(&mut out, total_payload_len as u32);
    out.extend_from_slice(&header);
    out.extend_from_slice(&final_payload);
    (out, ratio)
}

// ─── ERR ───────────────────────────────────────────────────────────────────

pub fn err(severity: u8, code: u8, ref_id: u16, message: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(severity);
    payload.push(0x01); // origin = server
    payload.push(code);
    push_u16(&mut payload, ref_id);
    push_str16(&mut payload, message);
    wrap_packet(PKT_ERR, payload)
}

// ─── PING ──────────────────────────────────────────────────────────────────

pub fn ping(sequence: u8) -> Vec<u8> {
    wrap_packet(PKT_PING, vec![sequence])
}

/// Construit un paquet EVENT (0x02) - Structure v0.5.0
pub fn event(node_id: u32, handler: &str, payload: &str) -> Vec<u8> {
    let mut data = Vec::new();
    push_u32(&mut data, 0); // SeqID Placeholder
    data.extend_from_slice(&[0u8; 32]); // Signature Placeholder
    
    push_u32(&mut data, node_id);
    push_str8(&mut data, handler);
    push_str16(&mut data, payload);
    wrap_packet(PKT_EVENT, data)
}

pub fn sync(checksum: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    push_u32(&mut payload, checksum);
    wrap_packet(PKT_SYNC, payload)
}

pub fn log_msg(severity: u8, message: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(severity);
    push_str16(&mut payload, message);
    wrap_packet(PKT_LOG, payload)
}

/// Instruction de diffusion multi-utilisateur (v0.6.0)
#[derive(Debug, Clone)]
pub struct BroadcastInstruction {
    pub scope: String, // "all", "others", "room", "direct"
    pub room_id: Option<String>,
    pub target_sid: Option<String>,
    pub patches: Vec<PatchOp>,
}
