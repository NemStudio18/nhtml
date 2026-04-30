# 🛰️ NHTML Technical Reference (v0.7.4)
**Reference document for the industrialized architecture.**

---

## 1. Overview
NHTML is an ultra-high performance "Server-Driven" web development framework. It transforms the browser into a binary rendering engine driven by a backend.

- **Gateway**: Rust server (Tokio) handling binary transport, HMAC security, and multiplexing.
- **Protocol**: NBPS (Native-HTML Binary Protocol) with adaptive Zstd compression.
- **Backend**: Your PHP applications (CGI or FastCGI/FPM).

---

## 2. General Architecture (v0.7.4)
The architecture relies on an **Orchestrator Gateway** acting as a secure bidirectional binary bridge.

```mermaid
graph TD
    Browser[Browser (bridge.js)] <-->|Binary NBPS + HMAC| Gateway[Rust Gateway]
    
    subgraph Backend
        Gateway <-->|FastCGI / TCP| FPM[PHP-FPM Pool]
        Gateway <-->|CGI / Stdout| PHP[CLI PHP App]
    end
    
    Gateway -->|Circuit Breaker| FPM
    Gateway <-->|Delta Sync| Browser
    Gateway -->|Monitoring| DevTools[DevTools Dashboard 8081]
    Gateway <-->|SQLite WAL| DB_Sessions[nhtml_sessions.db]
```

---

## 3. Communication & Resilience (v0.7.4)
- **FastCGI Load Balancing**: Intelligent dispatching (Round-Robin/Least-Connections) to PHP-FPM pools.
- **Circuit Breaker**: Automatic traffic cutoff to unstable backends to prevent cascading failures.
- **Delta Sync**: Intelligent state recovery after disconnection via patch replay (instead of a full snapshot).

---

## 4. Real-time Collaboration (Broadcasting)
The Gateway acts as a binary messaging server.
1. A session sends an event.
2. The backend processes and returns mutations for the sender AND broadcasting instructions.
3. The Gateway instantly routes packets to other concerned clients via the internal data bus.

---

## 5. Industrial Security
Every interaction is protected by:
- **HMAC-SHA256**: Guarantees packet origin and integrity.
- **Sequence ID**: Prevents replay attacks.
- **CSWH Protection**: WebSocket Origin validation.

---

## 6. Glossary
- **NID**: Textual identifier (string) dynamically mapped to a u16 binary ID.
- **FPM (FastCGI Process Manager)**: PHP process management system for production.
- **Delta History**: Transactional log of mutations allowing Delta Sync.
- **WAL (Write-Ahead Logging)**: SQLite logging mode optimized for concurrency.
