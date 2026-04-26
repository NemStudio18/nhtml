/// compiler/btree_builder.rs
/// Sérialise un arbre de NodeSpec en format B-TREE binaire.
/// Les attributs n- ont déjà été retirés par le compilateur.

use super::NodeSpec;

/// Sérialise un nœud et tous ses descendants (depth-first)
#[allow(dead_code)]
pub fn serialize_node(node: &NodeSpec) -> Vec<u8> {
    let mut buf = Vec::new();
    write_node(&mut buf, node);
    buf
}

#[allow(dead_code)]
fn write_node(buf: &mut Vec<u8>, node: &NodeSpec) {
    // ── HEADER ────────────────────────────────────────────────────────────
    buf.push(node.node_type);  // 0x01=element, 0x02=text

    // node_id (u16 big-endian)
    buf.push((node.id >> 8) as u8);
    buf.push((node.id & 0xFF) as u8);

    // tag_len + tag
    let tag = node.tag.as_bytes();
    buf.push(tag.len() as u8);
    buf.extend_from_slice(tag);

    // Calculer les flags
    let has_attrs    = !node.attrs.is_empty();
    let has_text     = !node.text.is_empty() && node.children.is_empty();
    let has_children = !node.children.is_empty();
    let listen_mask  = node.n_attrs.listen_mask();

    let mut flags: u8 = 0;
    if has_attrs    { flags |= 0x01; }
    if has_text     { flags |= 0x02; }
    if has_children { flags |= 0x04; }
    // listen_mask dans les bits 3-7
    flags |= (listen_mask & 0x1F) << 3;

    buf.push(flags);

    // ── ATTRS ─────────────────────────────────────────────────────────────
    if has_attrs {
        buf.push(node.attrs.len() as u8);
        for (key, val) in &node.attrs {
            let kb = key.as_bytes();
            let vb = val.as_bytes();
            buf.push(kb.len() as u8);
            buf.extend_from_slice(kb);
            buf.push((vb.len() >> 8) as u8);
            buf.push((vb.len() & 0xFF) as u8);
            buf.extend_from_slice(vb);
        }
    }

    // ── TEXT ──────────────────────────────────────────────────────────────
    if has_text {
        let tb = node.text.as_bytes();
        buf.push((tb.len() >> 8) as u8);
        buf.push((tb.len() & 0xFF) as u8);
        buf.extend_from_slice(tb);
    }

    // ── CHILDREN ──────────────────────────────────────────────────────────
    if has_children {
        buf.push((node.children.len() >> 8) as u8);
        buf.push((node.children.len() & 0xFF) as u8);
        for child in &node.children {
            write_node(buf, child);
        }
    }

    // ── END marker ────────────────────────────────────────────────────────
    buf.push(0xFF);
}
