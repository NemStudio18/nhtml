use wasm_bindgen::prelude::*;
use std::collections::HashMap;

// Imports from JS Bridge
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // DOM patching callbacks
    fn nhtml_set_text(nid: &str, text: &str);
    fn nhtml_add_class(nid: &str, cls: &str);
    fn nhtml_remove_class(nid: &str, cls: &str);
    fn nhtml_set_attr(nid: &str, attr: &str, val: &str);
    fn nhtml_set_style(nid: &str, prop: &str, val: &str);
    fn nhtml_show(nid: &str);
    fn nhtml_hide(nid: &str);
    fn nhtml_remove(nid: &str);
    fn nhtml_replace_inner(nid: &str, html: &str);

    // Local Actions callback
    fn nhtml_register_local_action(nid: &str, action_type: u8, trigger: u8, param: &str, flags: u8);
}

// ------------------------------------------------------------------
// State
// ------------------------------------------------------------------

#[wasm_bindgen]
pub struct NhtmlPolyfill {
    buffer: Vec<u8>,
    node_table: HashMap<u16, String>, // node_id -> n-id
}

#[wasm_bindgen]
impl NhtmlPolyfill {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        log("NHTML Polyfill v0.2.1 initialized.");
        Self {
            buffer: Vec::new(),
            node_table: HashMap::new(),
        }
    }

    #[wasm_bindgen]
    pub fn feed_bytes(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
        self.process_buffer();
    }

    fn process_buffer(&mut self) {
        // Simple assembler loop
        while self.buffer.len() >= 3 {
            let pkt_type = self.buffer[0];
            let length = if pkt_type == 0x07 { // B-TREE has 4-byte length
                if self.buffer.len() < 5 { break; }
                let l = u32::from_be_bytes([self.buffer[1], self.buffer[2], self.buffer[3], self.buffer[4]]) as usize;
                l + 5
            } else {
                let l = u16::from_be_bytes([self.buffer[1], self.buffer[2]]) as usize;
                l + 3
            };

            if self.buffer.len() < length {
                break; // Incomplete packet
            }

            let packet = self.buffer[..length].to_vec();
            self.buffer.drain(..length);

            self.parse_packet(&packet);
        }
    }

    fn parse_packet(&mut self, packet: &[u8]) {
        let pkt_type = packet[0];
        match pkt_type {
            0x01 => self.parse_hello(packet),
            0x02 => self.parse_patch(packet),
            0x04 => self.parse_bind(packet),
            0x07 => self.parse_btree(packet),
            _ => log(&format!("Unknown packet type: 0x{:02X}", pkt_type)),
        }
    }

    fn parse_hello(&mut self, _packet: &[u8]) {
        log("Received HELLO");
    }

    fn parse_btree(&mut self, packet: &[u8]) {
        if packet.len() < 13 { return; } // header size: type(1) + len(4) + comp(1) + uncomp_len(4) + crc(4)
        
        let mut cursor = 5; // Skip type and total packet length
        let compression_flag = packet[cursor]; cursor += 1;
        let uncompressed_len = u32::from_be_bytes([packet[cursor], packet[cursor+1], packet[cursor+2], packet[cursor+3]]) as usize; cursor += 4;
        let expected_crc = u32::from_be_bytes([packet[cursor], packet[cursor+1], packet[cursor+2], packet[cursor+3]]); cursor += 4;
        
        let compressed_data = &packet[cursor..];
        
        let decompressed = if compression_flag == 0x01 {
            // Zstd decompression using ruzstd
            use std::io::Read;
            let mut decoder = match ruzstd::StreamingDecoder::new(compressed_data) {
                Ok(d) => d,
                Err(e) => {
                    log(&format!("Erreur init zstd: {:?}", e));
                    return;
                }
            };
            
            let mut result = Vec::with_capacity(uncompressed_len);
            if let Err(e) = decoder.read_to_end(&mut result) {
                log(&format!("Erreur décompression zstd: {:?}", e));
                return;
            }
            result
        } else {
            compressed_data.to_vec()
        };
        
        // Verifier CRC32
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&decompressed);
        let actual_crc = hasher.finalize();
        
        if actual_crc != expected_crc {
            log(&format!("Erreur CRC32 B-TREE. Attendu: {:08X}, Reçu: {:08X}", expected_crc, actual_crc));
            return;
        }
        
        log(&format!("B-TREE décompressé et vérifié avec succès ! ({} bytes)", decompressed.len()));
        
        // TODO: Parser l'arbre DOM décompressé (JSON ou binaire selon la spec B-TREE)
        // et construire le DOM initial via des appels JS.
    }

    fn parse_bind(&mut self, packet: &[u8]) {
        if packet.len() < 5 { return; }
        
        let mut cursor = 3; // Skip header (1 byte type + 2 bytes length)
        
        let node_id = u16::from_be_bytes([packet[cursor], packet[cursor+1]]);
        cursor += 2;
        
        let nid = Self::read_str8(packet, &mut cursor);
        let _selector = Self::read_str8(packet, &mut cursor);
        
        let _listen_mask = packet[cursor]; cursor += 1;
        let _behavior_flags = packet[cursor]; cursor += 1;
        let _debounce = packet[cursor]; cursor += 1;
        
        let _handler = Self::read_str8(packet, &mut cursor);
        let _n_model = Self::read_str8(packet, &mut cursor);
        let _n_text = Self::read_str8(packet, &mut cursor);
        
        // Stocker le mapping node_id -> nid
        self.node_table.insert(node_id, nid.clone());
        
        // Extension v0.2.1 : Local Actions
        if cursor < packet.len() {
            let local_action_count = packet[cursor];
            cursor += 1;
            
            for _ in 0..local_action_count {
                if cursor + 2 > packet.len() { break; }
                let action_type = packet[cursor]; cursor += 1;
                let trigger_type = packet[cursor]; cursor += 1;
                let param = Self::read_str8(packet, &mut cursor);
                if cursor + 2 > packet.len() { break; }
                let flags = packet[cursor]; cursor += 1;
                let _threshold = packet[cursor]; cursor += 1;
                
                // Appel vers le JS Bridge
                nhtml_register_local_action(&nid, action_type, trigger_type, &param, flags);
            }
        }
    }

    fn read_str8(packet: &[u8], cursor: &mut usize) -> String {
        if *cursor >= packet.len() { return String::new(); }
        let len = packet[*cursor] as usize;
        *cursor += 1;
        if *cursor + len > packet.len() { return String::new(); }
        let s = String::from_utf8_lossy(&packet[*cursor..*cursor+len]).to_string();
        *cursor += len;
        s
    }

    fn parse_patch(&mut self, packet: &[u8]) {
        if packet.len() < 4 { return; }
        let mut cursor = 3; // Skip header
        
        let op_count = packet[cursor]; cursor += 1;
        
        for _ in 0..op_count {
            if cursor + 5 > packet.len() { break; }
            let op_type = packet[cursor]; cursor += 1;
            let target_id = u16::from_be_bytes([packet[cursor], packet[cursor+1]]); cursor += 2;
            let data_len = u16::from_be_bytes([packet[cursor], packet[cursor+1]]) as usize; cursor += 2;
            
            if cursor + data_len > packet.len() { break; }
            let data = &packet[cursor..cursor+data_len];
            cursor += data_len;
            
            // Resolve node ID
            let nid = match self.node_table.get(&target_id) {
                Some(id) => id,
                None => {
                    log(&format!("PATCH ignored: unknown node_id {}", target_id));
                    continue;
                }
            };
            
            // Apply operation
            match op_type {
                0x01 => { // SET_TEXT
                    let mut d_cursor = 0;
                    let text = Self::read_str16(data, &mut d_cursor);
                    nhtml_set_text(nid, &text);
                },
                0x04 => { // ADD_CLASS
                    let mut d_cursor = 0;
                    let class = Self::read_str16(data, &mut d_cursor);
                    nhtml_add_class(nid, &class);
                },
                0x05 => { // DEL_CLASS
                    let mut d_cursor = 0;
                    let class = Self::read_str16(data, &mut d_cursor);
                    nhtml_remove_class(nid, &class);
                },
                // Autres opérations à ajouter...
                _ => log(&format!("Unsupported PATCH op_type: {}", op_type)),
            }
        }
    }

    fn read_str16(data: &[u8], cursor: &mut usize) -> String {
        if *cursor + 2 > data.len() { return String::new(); }
        let len = u16::from_be_bytes([data[*cursor], data[*cursor+1]]) as usize;
        *cursor += 2;
        if *cursor + len > data.len() { return String::new(); }
        let s = String::from_utf8_lossy(&data[*cursor..*cursor+len]).to_string();
        *cursor += len;
        s
    }
}
