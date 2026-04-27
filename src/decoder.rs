/// decoder.rs — Décodeur universel du protocole NBPS v0.5.0
/// Permet de transformer un flux binaire opaque en structures lisibles (JSON).

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DecodedMessage {
    Hello {
        status: u8,
        session_id: String,
        secret: Vec<u8>,
        last_seq: u32,
    },
    Event {
        seq_id: u32,
        signature: Vec<u8>,
        node_id: u32,
        handler: String,
        payload: String,
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
        0x01 => { // HELLO (Server -> Client)
            if data.len() >= 5 {
                let _payload_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
                if data.len() >= 7 {
                    let status = data[5];
                    let sid_len = data[6] as usize;
                    if data.len() >= 7 + sid_len {
                        let session_id = String::from_utf8_lossy(&data[7..7+sid_len]).to_string();
                        let mut secret = Vec::new();
                        let mut last_seq = 0;
                        let mut cursor = 7 + sid_len;
                        if data.len() >= cursor + 32 {
                            secret = data[cursor..cursor+32].to_vec();
                            cursor += 32;
                        }
                        if data.len() >= cursor + 4 {
                            last_seq = u32::from_be_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]]);
                        }
                        DecodedMessage::Hello { status, session_id, secret, last_seq }
                    } else {
                        DecodedMessage::Unknown { opcode, len: data.len() }
                    }
                } else {
                    DecodedMessage::Unknown { opcode, len: data.len() }
                }
            } else {
                DecodedMessage::Unknown { opcode, len: data.len() }
            }
        },
        0x02 => { // EVENT (v0.5.0 Security)
            // [Type:1][Len:4][Seq:4][Sig:32][Payload...]
            // Payload = [NodeID:4][HLen:1][Handler:str][PLen:2][Payload:json]
            if data.len() >= 41 {
                let seq_id = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
                let signature = data[9..41].to_vec();
                let mut cursor = 41;
                
                if data.len() >= cursor + 4 {
                    let node_id = u32::from_be_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]]);
                    cursor += 4;
                    
                    if data.len() > cursor {
                        let h_len = data[cursor] as usize;
                        cursor += 1;
                        
                        if data.len() >= cursor + h_len + 2 {
                            let handler = String::from_utf8_lossy(&data[cursor..cursor+h_len]).to_string();
                            cursor += h_len;
                            
                            let p_len = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
                            cursor += 2;
                            
                            let payload = if data.len() >= cursor + p_len {
                                String::from_utf8_lossy(&data[cursor..cursor+p_len]).to_string()
                            } else {
                                "".to_string()
                            };
                            
                            return DecodedMessage::Event { seq_id, signature, node_id, handler, payload };
                        }
                    }
                }
                DecodedMessage::Unknown { opcode, len: data.len() }
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

                    if cursor + 2 > data.len() { break; }
                    let d_len = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
                    cursor += 2;

                    let mut value = String::new();
                    let op_name = match op_type_code {
                        0x01 => "SetText",
                        0x02 => "SetAttr",
                        0x03 => "DelAttr",
                        0x04 => "AddClass",
                        0x05 => "RemClass",
                        0x08 => "Remove",
                        0x09 => "SetStyle",
                        0x0A => "ReplaceInner",
                        0x0B => "AppendHtml",
                        0x0C => "ScrollTo",
                        0x0D => "Focus",
                        _ => "Unknown"
                    }.to_string();

                    if d_len > 0 && cursor + d_len <= data.len() {
                        value = String::from_utf8_lossy(&data[cursor..cursor+d_len]).to_string();
                        cursor += d_len;
                    }

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
        0x09 => { // PING
            if data.len() >= 6 {
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

