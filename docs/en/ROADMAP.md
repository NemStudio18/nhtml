# 🗺️ NHTML Roadmap (v0.4.0)

## ✅ Phase 1 — Foundations & Glow (Completed)
- [x] NHTML Glow, Packet HUD

## ✅ Phase 2 — Industrial Transport (Completed)
- [x] SQLite Sessions, PHP Log Bridge, Protocol v0.3.1, Auto-PHP Supervisor

## ✅ Phase 3 — DevTools Pro (Completed)
- [x] Network Monitor, Node Inspector, State Diff Viewer, Handler Tracer, Session Comparator

## ✅ Phase 4 — Optimization & Performance (Completed)
- [x] **Zstd Compression**: Compressed B-TREE + client decompression (local fzstd, zero CDN)
- [x] **Benchmark CLI**: `nhtml bench <file>`
- [x] **Auto-Injection Bridge**: `bridge.js` + `fzstd.js` automatically injected into `.nhtml`
- [x] **Configurable Ports**: `--ws-port`, `--php-port`, `--port` (devtools)
- [x] **Example Pyramid**: counter → todo → live-form → style-lab
- [x] **Multi-Platform Packaging**: Win/Linux/Mac binaries via GitHub Actions.

## 🔵 Phase 5 — Public Release & Ecosystem (Active)
- [x] **Unified Monorepo**: Single repository containing Rust, PHP SDK, and JS Bridge.
- [x] **PHP-WASM Fallback**: "Zero-Server" mode integrated into the JS Bridge (Local & CDN-free).
- [x] **`nhtml.config.toml`**: Declarative configuration for ports and paths.
- [x] **Reverse Proxy Docs**: Nginx/Apache configs ready for copy-paste.
- [ ] **Public Landing Page**: Interactive documentation site on GitHub Pages.
- [ ] **Community SDKs**: Python and Go ports of the NHTML protocol.

---
> **Target Architecture (Production)**: `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools never exposed.
