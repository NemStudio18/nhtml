# 🛰️ NHTML/// proto.rs — Binary serialization for Nhtml v0.7.1
**Performance & Collaboration Edition**

## 1. Universal Packet Structure
Each binary packet starts with a **5-byte Header**:
`[Type: u8] [Length: u32]` (Big-Endian)

---

## 2. Packet Types (Header Type)

| Code | Name | Direction | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bi-directional | Handshake, Secret & SeqID sync. |
| `0x02` | **EVENT** | Client -> Srv | HMAC-SHA256 signed interaction. |
| `0x03` | **PATCH** | Srv -> Client | Atomic DOM mutation instructions. |
| `0x04` | **BIND** | Srv -> Client | Local Actions registration. |
| `0x07` | **B-TREE** | Srv -> Client | Full DOM Snapshot (Zstd Compressed). |
| `0x08` | **PUSH_PATCH**| Client -> Srv | Local Patch relay (Zero-Server Mode). |
| `0x09` | **PING/PONG** | Bi-directional | Keep-Alive & Heartbeat. |

---

##[command(about = "NHTML Gateway - NBPS v0.7.0", long_about = None)]

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

*   **scope**: `all` (everyone), `others` (everyone except sender), `group:X` (future).

---

## 5. Performance & Transport
- **FastCGI Client**: The Gateway implements a native FastCGI client to talk to PHP-FPM pools (Default Port 9000).
- **Zstd Compression**: Dictionary-based Zstd usage for massive B-TREE snapshots.
- **Binary Stream**: All communications are binary (except the internal Gateway <-> PHP dialogue which uses JSON streamed over stdin/stdout).
