/// compiler/btree_builder.rs
/// Sérialise un arbre de NodeSpec en format B-TREE binaire.
/// Les attributs n- ont déjà été retirés par le compilateur.

use super::NodeSpec;

/// Sérialise un nœud et tous ses descendants (depth-first)
#[allow(dead_code)]
pub fn serialize_node(node: &NodeSpec) -> Vec<u8> {
    let mut buf = Vec::with_capacity(1024);
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
    let tag_len = tag.len().min(255);
    buf.push(tag_len as u8);
    buf.extend_from_slice(&tag[..tag_len]);

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
        buf.push(node.attrs.len().min(255) as u8);
        for (key, val) in node.attrs.iter().take(255) {
            let kb = key.as_bytes();
            let vb = val.as_bytes();
            let kb_len = kb.len().min(255);
            buf.push(kb_len as u8);
            buf.extend_from_slice(&kb[..kb_len]);
            
            let mut vb_len = vb.len();
            if vb_len > 65535 {
                vb_len = 65535;
                while !val.is_char_boundary(vb_len) && vb_len > 0 { vb_len -= 1; }
            }
            buf.push((vb_len >> 8) as u8);
            buf.push((vb_len & 0xFF) as u8);
            buf.extend_from_slice(&vb[..vb_len]);
        }
    }

    // ── TEXT ──────────────────────────────────────────────────────────────
    if has_text {
        let mut tb_len = node.text.len();
        if tb_len > 65535 {
            tb_len = 65535;
            while !node.text.is_char_boundary(tb_len) && tb_len > 0 { tb_len -= 1; }
        }
        let tb = node.text.as_bytes();
        buf.push((tb_len >> 8) as u8);
        buf.push((tb_len & 0xFF) as u8);
        buf.extend_from_slice(&tb[..tb_len]);
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
