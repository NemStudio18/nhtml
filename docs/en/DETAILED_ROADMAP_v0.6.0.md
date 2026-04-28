# 🗺️ Detailed Roadmap NHTML v0.6.0 "Global Connect"

This roadmap defines the precise actions required to transform NHTML into an ultra-high-performance, collaborative production solution.

---

## 🚀 Phase 6.1: High-Performance Infrastructure (FastCGI)
**Goal**: Eliminate PHP fork overhead and stabilize industrial backends.

### 1. Persistent Connection Pool
- [x] **Action**: Implement a socket pool management system in `src/socket/mod.rs`.
- [x] **Action**: Manage `Keep-Alive` to reuse FastCGI connections between multiple events in the same session.
- [x] **Action**: Add a configurable socket timeout in `nhtml.config.toml`.

### 2. Unix Socket Support (Linux/macOS)
- [x] **Action**: Extend the binary to accept Unix socket paths (e.g., `/var/run/php/php8.2-fpm.sock`) in addition to TCP.
- [x] **Action**: Auto-detect mode (Stream vs Unix) based on the FPM address prefix.

---

## 📡 Phase 6.2: Real-Time Collaboration (Broadcasting)
**Goal**: Enable seamless multi-user experiences.

### 1. Scoped Routing Engine
- [x] **Done**: Sender identification via `SenderSID`.
- [x] **Action**: Optimize the broadcasting loop to avoid unnecessary binary payload clones (using `Arc<Vec<u8>>`).
- [x] **Action**: Add support for "Rooms" (session groups) to limit broadcasting to a subset of users.

### 2. Extended PHP SDK
- [x] **Action**: Add `Nhtml::joinRoom()`, `Nhtml::leaveRoom()` and `Nhtml::broadcastInRoom()` methods to the PHP SDK.
- [x] **Action**: Allow simultaneous sending of a private patch AND a public broadcast in the same JSON response.

---

## 🛡️ Phase 6.3: Stability & Error Management
**Goal**: Zero crashes, 100% visibility.

### 1. "No-Panic" Refactoring
- [x] **Done**: Introduction of `GatewayError`.
- [x] **Action**: Replace the last `expect()` and `unwrap()` in `main.rs` and `supervisor.rs` with proper error propagation.
- [ ] **Action**: Improve logging of FastCGI errors (Timeout, Connection Refused) to display them in DevTools.

### 2. Auto-Recovery (Healthchecks)
- [ ] **Action**: The Supervisor should attempt to restart the backend if a systematic crash is detected.
- [ ] **Action**: Display a visual alert in the browser via a special `LOG` packet if the backend is unreachable.

---

## ☁️ Phase 6.4: Cloud Connectivity & Deployment
**Goal**: Facilitate secure remote access.

### 1. Cloud Tunneling Integration (Optional)
- [ ] **Action**: Explore lightweight integration of a `cloudflared` or `ngrok` binary to expose the local Gateway externally with a single command.
- [ ] **Action**: Add an `nhtml tunnel` command to the CLI.

### 2. Production Documentation
- [ ] **Action**: Create exemplary Nginx and Apache configuration files including WebSocket handling (`Upgrade: websocket`).
- [ ] **Action**: Security hardening guide (Fail2Ban, Rate Limiting).

---

> **Current Status**: The core FastCGI architecture and Broadcasting are functional. The immediate priority is **pool stability** and **refactoring critical errors**.
