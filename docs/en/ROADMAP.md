# 🗺️ NHTML Roadmap (v0.6.0)

## ✅ Phase 5 — Security & Stability (Completed)
- [x] **HMAC-SHA256 Signatures**: Binary authentication for client packets.
- [x] **Sequence ID Synchronization**: Integrated anti-replay protocol.
- [x] **WASM Persistence**: IDBFS support for Messenger/Settings.
- [x] **Responsive Showcase**: Adaptive Desktop/Mobile interface.

## 🔵 Phase 6 — Performance & Collaboration (Active)
- [x] **FastCGI Client (Rust)** : Direct bridge to PHP-FPM pool (zero fork overhead).
- [x] **Scoped Broadcasting** : Real-time multi-user synchronization (`all`, `others`).
- [x] **Professional Error Handling** : `GatewayError` system for 100% server stability.
- [ ] **Connection Pooling** : Reuse FPM sockets (Keep-alive).
- [ ] **Auto-Tunneling** : Optional Cloudflare Tunnel / Ngrok integration for Cloud Home.
- [ ] **Community SDKs**: Python and Go ports of the NHTML protocol.

---
> **Target Architecture (Production)**: `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools never exposed.

---
> **Target Architecture (Production)**: `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools never exposed.
