# 🛰️ NHTML Technical Reference (v0.4.0)
**Technical reference for the industrialized architecture.**

---

## 1. Project Overview
NHTML (Native-HTML) is a "Server-Driven" web development framework designed to move all business logic and application state to the server side while offering responsiveness close to native clients.

- **Objective**: Eliminate frontend JS complexity by turning the browser into a simple binary rendering engine driven by a backend (PHP/Rust).
- **Gateway**: Rust server (Tokio, Axum) managing transport, persistence, and auto-injection.
- **Protocol**: NBPS (Native-HTML Binary Protocol) optimized with native Zstd compression.

---

## 2. General Architecture
The architecture relies on an **Orchestrator Gateway** acting as a bidirectional binary bridge.

```mermaid
graph TD
    Browser[Browser (bridge.js)] <-->|NBPS Binary + Zstd| Gateway[Gateway Rust]
    Gateway <-->|JSON/HTTP| PHP[PHP App (SDK)]
    
    Gateway -->|Monitoring| DevTools[DevTools Dashboard 8081]
    Gateway <-->|SQLite| DB_Sessions[nhtml_sessions.db]
    PHP <-->|SQLite| DB_App[app.db]
```

---

## 3. Directory Structure (v0.4.0)
- **`/src/`**: Core Rust system (Gateway & CLI).
- **`/static/`**: DevTools Dashboard templates.
- **`/assets/js/`**: Client bridge and decompression polyfills.
- **`/sdk/php/`**: Official SDK for the PHP backend.
- **`/examples/`**: Concrete use cases (Counter, TodoList...).

---

## 4. NBPS v0.4.0 Protocol
The protocol is optimized for bandwidth (Zstd) and reliability (Checksums).

- **Universal Header (5 bytes)**: `[Type: u8] [Length: u32]`.
- **OpCodes v0.4.0**:
    - `0x01` (HELLO): Initial handshake and session ID.
    - `0x02` (EVENT): Client -> server interaction (click, input).
    - `0x03` (PATCH): Atomic DOM mutations (setText, setStyle, focus...).
    - `0x05` (SYNC): DOM integrity check via checksum.
    - `0x07` (B-TREE): Compressed snapshot of full state.
    - `0x09` (PING): Connection maintenance heartbeat.
    - `0x7F` (ERROR): Structured binary error report.

---

## 5. Auto-Injection Flow
NHTML v0.4.0 radically simplifies deployment. The Gateway automatically injects the bridge and polyfills into any served `.nhtml` file.

1. **Request**: The client requests `index.nhtml`.
2. **Interception**: The Gateway reads the file, injects `<script>` tags for `bridge.js` and `fzstd.js` before the `</head>` tag.
3. **Activation**: The bridge automatically starts the WebSocket connection to the Gateway.

---

## 6. Monitoring & Time Travel
Every mutation is historicalized in `nhtml_sessions.db`.
The Dashboard (port 8081) allows you to:
- **Visualize** real-time flows (Network Monitor).
- **Replay** a session step-by-step (Time Travel).
- **Compare** the state of two distinct sessions.
- **Audit** the exact latency of PHP responses.

---

## 7. Glossary
- **NID**: Textual identifier (string) dynamically mapped to a binary u16 ID.
- **Patch Glow**: Visual feedback signaling a reactive update.
- **Zero-JS**: Concept where no business JavaScript code is written by the developer.
