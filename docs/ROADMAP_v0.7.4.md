# 🗺️ NHTML Roadmap v0.7.4 — Performance & Observability

This document outlines the planned improvements for the next minor release cycle.

## 🏗️ Core Architecture & Infrastructure
- [x] **SQLite WAL Mode**: Move session database to WAL (Write-Ahead Logging) for better concurrent performance.
- [x] **Circuit Breaker for FPM**: Implement a circuit breaker in `socket/mod.rs` to stop trying FPM if the backend is persistently failing.
- [x] **Exponential Backoff**: Better reconnection strategy in `bridge.js` with exponential backoff and jitter.
- [x] **nhtml check command**: A new pre-deployment command that verifies all routes and config security before going live.

## ⚡ Performance (v1.0 Milestone)
- [ ] **Delta Sync**: At reconnection, only send the patches missed since last state instead of a full B-TREE snapshot.
- [ ] **Zstd Level Tuning**: Fine-tune Zstd compression levels based on packet size (Gateway vs Bridge).
- [ ] **PHP HMR**: Hot Module Replacement for PHP logic (reload parts of the B-TREE without full session refresh).

## 📊 Observability & Metrics
- [ ] **Prometheus Exporter**: Native `/metrics` endpoint with latency histograms, active sessions, and compression ratios.
- [ ] **OpenTelemetry**: Distributed tracing support to track a request from `bridge.js` through the Gateway to the PHP backend.
- [ ] **Audit Trail**: Optional secure log of all sensitive state changes (input values, session logins).

## 🛠️ Developer Experience (DX)
- [x] **Unified SDK Interface**: Ensure all SDKs (Python, Go, Node) have 100% parity with the PHP SDK.
- [ ] **Better Error Overlays**: Show PHP stack traces directly in the browser during `--dev` mode.
- [ ] **SSE Fallback**: Official support for Server-Sent Events for environments where WebSockets are blocked.

---
*Last Updated: 2026-04-30 — NHTML v0.7.3-stable*
