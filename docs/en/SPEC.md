# 🛰️ NHTML Protocol Specification (NBPS) v0.4.0
**Single Source of Truth**

## 1. Universal Packet Structure
Each binary packet starts with a **5-byte Header**:
`[Type: u8] [Length: u32]` (Big-Endian)

The `Length` field represents the total size of the payload (excluding the header).

---

## 2. Packet Table (NBPS v0.4.0)

| Code | Name | Direction | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bi-directional | Handshake, Session Sync & Version. |
| `0x02` | **EVENT** | Client -> Srv | User interaction (Click, Input). |
| `0x03` | **PATCH** | Srv -> Client | Atomic DOM mutation instructions. |
| `0x04` | **BIND** | Srv -> Client | Registration of Local Actions & Handlers. |
| `0x05` | **SYNC** | Srv -> Client | Integrity check (Checksum comparison). |
| `0x07` | **B-TREE** | Srv -> Client | Complete DOM snapshot with NIDs (Mapping included). |
| `0x09` | **PING/PONG** | Bi-directional | Keep-Alive & Heartbeat (0xFF = Force Reload). |
| `0x10` | **LOG** | Srv -> Client | Relay of backend console logs. |
| `0x7F` | **ERR** | Srv -> Client | Structured binary error report. |

---

## 3. DOM Operations (OpTypes in 0x03 PATCH)

Structure of a mutation in a PATCH packet:
`[TargetID: u16] [OpType: u8] [Version: u32] [DataLen: u16] [Data: bytes]`

| Code | Operation | Parameter(s) |
|:---|:---|:---|
| `0x01` | **SET_TEXT** | string16 |
| `0x02` | **SET_ATTR** | string8 (key) + string16 (val) |
| `0x03` | **DEL_ATTR** | string8 (key) |
| `0x04` | **ADD_CLASS** | string16 (class) |
| `0x05` | **DEL_CLASS** | string16 (class) |
| `0x06` | **INSERT_BEFORE**| u16 (refId) + string16 (HTML) |
| `0x07` | **INSERT_AFTER** | u16 (refId) + string16 (HTML) |
| `0x08` | **REMOVE** | (none) |
| `0x09` | **SET_STYLE** | string8 (prop) + string16 (val) |
| `0x0A` | **REPLACE_INNER**| string16 (HTML) |
| `0x0B` | **APPEND_HTML** | string16 (HTML) |
| `0x0C` | **SCROLL_TO** | (none) |
| `0x0D` | **FOCUS** | (none) |

---

## 4. Gateway ↔ Backend Interface (Generic JSON)

The Rust Gateway is completely generic. It maps `nid` (strings) to `node_id` (u16) dynamically.

### Request (Gateway -> PHP/CGI)
The Gateway transmits the event via environment variables (CGI) or via a structured JSON payload:
```json
{
  "nhtml_event": "click",
  "node_id": 12,
  "session_id": "uuid-...",
  "payload": { ... }
}
```

### Response (PHP -> Gateway)
The PHP SDK uses `snake_case` operation names to generate patches.
```json
{
  "patch": [
    { "op": "set_text", "nid": "counter", "val": "42" },
    { "op": "add_class", "nid": "header", "val": "active" }
  ]
}
```

---

## 5. SYNC Mechanism (Checksum)
Periodically, the server sends `0x05 [Checksum: u32]`.
1. The client calculates its `local_checksum` (ID + Value length).
2. If `local_checksum != server_checksum`, the client issues a `HELLO` (0x01).
3. The server responds with a `B-TREE` (0x07) to restore the state.

---

## 6. B-TREE Format (Mapping v0.4.0)
Each node in the B-TREE now includes its `tag` (NID string).
`[Count: u16] { [ID: u16] [Version: u32] [TagLen: u8] [Tag: str8] [ValLen: u16] [Val: str16] }`

---

## 7. Performance & Optimization
- **Zstd** is used by default on `B-TREE` packets.
- **IIFE Guard** in `bridge.js` to prevent double initialization.
- **Node Caching** on the client side for ultra-fast mutations (< 1ms).
