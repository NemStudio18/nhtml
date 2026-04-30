# 🛰️ NHTML Technical Reference (v0.7.3)
**Technical reference for the industrialized architecture.**

---

## 1. Overview
NHTML is an ultra-high-performance "Server-Driven" web development framework. It transforms the browser into a binary rendering engine driven by a backend.

- **Gateway**: Rust server (Tokio) managing binary transport, HMAC security, and multiplexing.
- **Protocol**: NBPS (Native-HTML Binary Protocol) with native Zstd compression.
- **Backend**: Your PHP applications (CGI or FastCGI/FPM).

---

## 2. General Architecture (v0.7.3)
The architecture relies on an **Orchestrator Gateway** acting as a secure bidirectional binary bridge.

```mermaid
graph TD
    Browser[Browser (bridge.js)] <-->|NBPS Binary + HMAC| Gateway[Gateway Rust]
    
    subgraph Backend
        Gateway <-->|FastCGI / TCP| FPM[PHP-FPM Pool]
        Gateway <-->|CGI / Stdout| PHP[PHP CLI App]
    end
    
    Gateway -->|Broadcast| Others[Other Sessions]
    Gateway -->|Monitoring| DevTools[DevTools Dashboard 8081]
    Gateway <-->|SQLite| DB_Sessions[nhtml_sessions.db]
```

---

## 3. High-Performance Communication
NHTML v0.7.3 implements native **FastCGI** support with load balancing.
Instead of launching a PHP process for every click, the Gateway maintains open sockets to a PHP-FPM pool, reducing latency to < 5ms.

---

## 4. Real-Time Collaboration (Broadcasting)
The Gateway acts as a binary messaging server.
1. A session sends an event.
2. The backend processes it and returns mutations for the sender AND broadcasting instructions.
3. The Gateway instantly routes packets to other relevant clients.

---

## 5. Industrial Security (v0.7.3)
Every interaction is protected by:
- **HMAC-SHA256**: Guarantees packet origin and integrity.
- **Sequence ID**: Prevents any replay attacks.

---

## 6. Glossary
- **NID**: Textual identifier (string) dynamically mapped to a binary u16 ID.
- **FPM (FastCGI Process Manager)**: PHP process management system for production.
- **Zero-JS**: Concept where no business JavaScript code is written by the developer.
