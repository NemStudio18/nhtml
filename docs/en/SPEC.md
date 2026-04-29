# 🛰️ NHTML Protocol Specification (NBPS) v0.7.3
**Security & Stability Edition**

## 1. Universal Packet Structure
Each binary packet starts with a **5-byte Header**:
`[Type: u8] [Length: u32]` (Big-Endian)

---

## 2. Packet Types (Header Type)

| Code | Name | Direction | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bi-directional | Handshake, HMAC Secret & SeqID synchronization. |
| `0x02` | **EVENT** | Client -> Srv | User interaction (e.g. click, input) signed via HMAC-SHA256. |
| `0x03` | **PATCH** | Srv -> Client | Atomic DOM mutation instructions (add, modify, delete). |
| `0x07` | **B-TREE** | Srv -> Client | Full DOM Snapshot (Zstd Compressed). |
| `0x08` | **PUSH_PATCH**| Client -> Srv | Local Patch relay (Zero-Server Mode / Real-time collaboration). |
| `0x09` | **PING/PONG** | Bi-directional | Keep-Alive & Heartbeat mechanism. |
| `0x10` | **LOG** | Srv -> Client | System logs broadcasted to client DevTools. |

---

## 3. Payload Details

### 0x01 - HELLO
`[Type:1][Len:4][Status:1][SidLen:1][SessionId...][Secret:32][LastSeq:4]`

### 0x02 - EVENT
`[Type:1][Len:4][SeqId:4][Signature:32][NodeID:4][HLen:1][Handler:str][PLen:2][Payload:json]`

### 0x03 - PATCH
`[Type:1][Len:4][OpCount:2]` followed by operation list:
`[TargetID:2][OpTypeCode:1][Version:4][DataLen:2][Value...]`

### 0x07 - B-TREE
`[Type:1][Len:4][Compression:1][OrigLen:4][Checksum:4][NodeCount:2][ZstdData...]`

### 0x08 - PUSH_PATCH
Identical to PATCH (`0x03`) but emitted by the client, limited to a maximum of 64 operations per message to prevent render thread overload.

### 0x10 - LOG
`[Type:1][Len:4][Severity:1][MsgLen:2][Message...]`

---

## 4. Gateway ↔ Backend (PHP) Interface Contract

The Gateway communicates with PHP via **CGI (Standard)** or **FastCGI (Performance)**.

### PHP Response (JSON Format)
The PHP SDK can return a simple array of patches or a structured object to include broadcasting instructions.

```json
{
  "patch": [
    { "op": "set_text", "nid": "status", "val": "Connected" }
  ],
  "broadcast": {
    "scope": "others",
    "patch": [
      { "op": "append_html", "nid": "logs", "val": "<li>User-42 clicked!</li>" }
    ]
  }
}
```

*   **scope**: `all` (everyone), `others` (everyone except sender), `room:X` (localized group), `direct:X` (private message).

---

## 5. Performance, Security & Transport
- **CSWH Security**: Strict origin validation via whitelist (`allowed_origins` in `nhtml.config.toml`).
- **Rate Limiting**: O(1) LRU-based cache per IP address (Default limit: 30 events/sec to mitigate DoS).
- **Zstd Compression**: Dictionary-based Zstd usage for massive B-TREE snapshots.
- **Binary Stream**: All communications are binary, using strict UTF-8 decoding for string integrity.
