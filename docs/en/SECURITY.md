# 🛡️ Security & Technical Limitations (NHTML)

This document details the critical security points and architectural limits of the NHTML system.

---

## 1. Authentication & Integrity (v0.7.3 ✅)
### Implemented Solution
Mandatory cryptographic security layer is enabled by default:
- **HMAC-SHA256 Signatures**: Every event sent by the client (`EVENT`) is signed with a 32-byte secret key negotiated during the handshake. The Gateway immediately rejects any falsified frames.
- **Sequence ID (Anti-Replay)**: An incremental counter is maintained per session. The Gateway only accepts packets with a `SeqID` higher than the previous one, making replay attacks impossible.
- **Marvin Attack**: *Update (v0.7.3):* This vulnerability (RUSTSEC-2023-0071) has been officially resolved in the latest dependency tree update.

---

## 2. High Performance & FastCGI (v0.7.3 ✅)
### The Risk
In classic CGI mode, the Gateway launches a PHP process for each event. While simple, this can be exploited to saturate the CPU (DoS).
### Solution
- **FastCGI (PHP-FPM)**: The Gateway maintains persistent connections to a PHP worker pool. This drastically reduces process creation overhead and allows resource limiting at the FPM server level.
- **Rate Limiting (v0.7.1+)**: Limiting the number of events per second per IP at the Gateway level via `[security.rate_limit]`.
- **Native TLS (v0.7.3+)**: WSS (WebSocket Secure) and HTTPS (`min_version = "1.3"`) support directly in the Gateway without requiring a reverse proxy.

---

## 3. WASM Mode Limitations (Zero-Server)
WASM mode has intrinsic browser security limits:
- **Total Isolation**: Each WASM client is an "island." There is **no synchronization possible** between two users on a static page (GitHub Pages), as there is no central server to arbitrate state.
- **Sandbox**: PHP-WASM cannot open outbound network sockets. Only **local SQLite** (via IDBFS) is supported.
- **Usage**: This mode is reserved for "Offline-First" tools or static demonstrations.

---

## 4. DevTools Exposure
NHTML DevTools (`port 8081`) expose all business flows:
- **Risk**: **Never** expose port 8081 to the public Internet in production.
- **Recommendation**: Use an SSH tunnel or VPN exclusively for remote access.

---

## 5. Future Security Roadmap
- **Client Session Storage**: Migration of the session ID from `localStorage` to `sessionStorage` for automatic expiration on tab close.
- **Prometheus Monitoring**: Full integration of rejected attack metrics to Grafana.
