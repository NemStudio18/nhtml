/// decoder.rs — Décodeur universel du protocole NBPS v0.2.2
/// Permet de transformer un flux binaire opaque en structures lisibles (JSON).

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DecodedMessage {
    Hello {
        version: u32,
        session_id: String,
    },
    Event {
        node_id: u32,
    },
    Patch {
        op_count: u16,
        ops: Vec<DecodedOp>,
    },
    BTree {
        total_len: u32,
        compression: u8,
        orig_len: u32,
        checksum: u32,
        node_count: u16,
    },
    Sync {
        version: u32,
    },
    Ping {
        sequence: u8,
    },
    Log {
        severity: u8,
        message: String,
    },
    Unknown {
        opcode: u8,
        len: usize,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedOp {
    pub target_id: u16,
    pub op_type: String,
    pub version: u32,
    pub value: String,
}

pub fn decode(data: &[u8]) -> DecodedMessage {
    if data.is_empty() {
        return DecodedMessage::Unknown { opcode: 0, len: 0 };
    }

    let opcode = data[0];
    match opcode {
        0x01 => { // HELLO
            if data.len() >= 5 {
                let payload_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
                if data.len() >= 5 + payload_len {
                    let version = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
                    let sid_len = data[9] as usize;
                    let session_id = String::from_utf8_lossy(&data[10..10+sid_len]).to_string();
                    DecodedMessage::Hello { version, session_id }
                } else {
                    DecodedMessage::Unknown { opcode, len: data.len() }
                }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x02 => { // EVENT
            if data.len() >= 5 {
                let node_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                DecodedMessage::Event { node_id }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x03 => { // PATCH
            if data.len() >= 7 {
                let _payload_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                let op_count = u16::from_be_bytes([data[5], data[6]]);
                let mut ops = Vec::new();
                let mut cursor = 7;

                for _ in 0..op_count {
                    if cursor + 7 > data.len() { break; }
                    let target_id = u16::from_be_bytes([data[cursor], data[cursor+1]]);
                    let op_type_code = data[cursor+2];
                    let version = u32::from_be_bytes([data[cursor+3], data[cursor+4], data[cursor+5], data[cursor+6]]);
                    cursor += 7;

                    let mut value = String::new();
                    let mut op_name = format!("Op:{}", op_type_code);

                    if op_type_code == 0x01 || op_type_code == 0x0A { // SetText or ReplaceInner
                        op_name = if op_type_code == 0x01 { "SetText" } else { "ReplaceInner" }.to_string();
                        if cursor + 2 <= data.len() {
                            let len = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
                            cursor += 2;
                            if cursor + len <= data.len() {
                                value = String::from_utf8_lossy(&data[cursor..cursor+len]).to_string();
                                cursor += len;
                            }
                        }
                    }
                    // TODO: Ajouter les autres types d'Op (0x02 SetAttr, etc.)

                    ops.push(DecodedOp {
                        target_id,
                        op_type: op_name,
                        version,
                        value,
                    });
                }

                DecodedMessage::Patch { op_count, ops }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x07 => { // BTREE
            if data.len() >= 16 {
                let _total_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                let compression = data[5];
                let orig_len = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
                let checksum = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
                let node_count = u16::from_be_bytes([data[14], data[15]]);
                DecodedMessage::BTree { total_len: _total_len, compression, orig_len, checksum, node_count }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x09 => { // PING (was 0x06)
            if data.len() >= 5 {
                DecodedMessage::Ping { sequence: data[5] }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x10 => { // LOG
            if data.len() >= 8 {
                let _payload_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                let severity = data[5];
                let msg_len = u16::from_be_bytes([data[6], data[7]]) as usize;
                if data.len() >= 8 + msg_len {
                    let message = String::from_utf8_lossy(&data[8..8+msg_len]).to_string();
                    DecodedMessage::Log { severity, message }
                } else {
                    DecodedMessage::Unknown { opcode, len: data.len() }
                }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        _ => DecodedMessage::Unknown { opcode, len: data.len() }
    }
}
