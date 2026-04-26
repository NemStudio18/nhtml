# 🏗️ NHTML Architecture

## 1. Overview
NHTML (Next-generation HyperText Markup Language) is a server-driven UI framework designed for high-performance, real-time web applications. It eliminates the need for client-side JavaScript by moving the application state and business logic to the server.

## 2. Core Components

### 🛰️ The Gateway (Rust)
The Gateway is the brain of the system. It:
- Compiles `.nhtml` files into a binary **B-TREE**.
- Manages WebSocket connections via the **NBPS** protocol.
- Proxies events to the backend (PHP/CGI).
- Calculates DOM diffs and sends atomic patches to the client.

### 🔌 The Bridge (JavaScript)
A lightweight (~25KB) script that runs in the browser. It:
- Establishes the WebSocket connection.
- Receives the B-TREE and renders the initial DOM.
- Intercepts user interactions (`n-click`, etc.) and sends them as binary events.
- Receives binary patches and applies them directly to the DOM using high-speed caching.

### 🐘 The Backend (PHP SDK)
The application logic. It:
- Receives event payloads from the Gateway.
- Processes business logic (database, sessions).
- Returns a list of high-level DOM operations (`setText`, `addClass`, etc.).

## 3. Communication Flow (NBPS)
1. **Handshake**: Client sends `HELLO`. Server responds with `B-TREE` (compressed snapshot).
2. **Interaction**: User clicks a button -> `EVENT` packet (3-10 bytes).
3. **Execution**: Gateway runs PHP -> PHP returns JSON patches.
4. **Mutation**: Gateway converts JSON to binary `PATCH` packets -> Bridge applies them.

## 4. Security
- **No JS Exposure**: Business logic never leaves the server.
- **Strict ID Mapping**: Clients can only interact with nodes that have an `n-id` or registered handler.
- **Binary Obfuscation**: The protocol is binary, making reverse engineering harder than standard REST/JSON APIs.
