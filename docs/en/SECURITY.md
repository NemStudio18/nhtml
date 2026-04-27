# 🛡️ Security & Technical Limitations (NHTML)

This document details the critical security points and current architectural limitations of the NHTML system. These points are the primary development focus for version **v0.5.0**.

---

## 1. Session Management & Race Conditions
### The Problem
In **Dedicated Mode (Rust)**, the Gateway is asynchronous. If a user triggers multiple events simultaneously (e.g., rapid clicks), the Gateway may launch several PHP processes in parallel.
While PHP uses file locking (`session_lock`) to force requests to execute sequentially, if the PHP code manipulates persistent state (Database, Files) without transactions, inconsistencies can occur.

### Planned Solution (v0.5.0)
- **Atomic Sequence ID**: Every mutation will be associated with a state version number. If the Gateway receives a response based on an obsolete version, it will be rejected to prevent data overwrites.

---

## 2. Protocol Integrity (NBPS)
### Injection Risk
The NBPS binary protocol currently lacks a per-packet authentication mechanism. An attacker capable of connecting to the WebSocket could inject their own binary frames to manipulate the victim's DOM (Binary XSS).

### Planned Solution (v0.5.0)
- **HMAC Signing**: Every packet sent by the Gateway will be signed with a shared secret key. The `bridge.js` will verify the signature before applying any mutation.

---

## 3. WASM Mode Limitations (Zero-Server)
The WASM mode is revolutionary but has intrinsic limitations due to browser security models:
- **Total Isolation**: Every WASM client is an "island." There is **no possible synchronization** between two users on a static page (GitHub Pages), as there is no central server to arbitrate the state.
- **Sandbox**: PHP-WASM cannot open network sockets (TCP/UDP) to the outside. Connections to remote databases (MySQL, PostgreSQL) are impossible. Only **local SQLite** (in RAM or IndexedDB) is supported.
- **Usage**: This mode is reserved for "Offline-First" tools, calculators, or demonstrations.

---

## 4. DevTools Exposure
NHTML DevTools (`port 8081`) are a powerful but dangerous diagnostic tool:
- **Full Visibility**: They expose all business flows and data structures.
- **Risk**: **Never** expose port 8081 to the public Internet in production.
- **Recommendation**: Exclusively use an SSH tunnel to access them remotely.
