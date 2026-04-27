# 🚀 NHTML Getting Started Guide (v0.4.0)
**Welcome to the Web without JavaScript. Native Performance, Server Logic.**

This guide will walk you through installation to your first reactive interaction.

---

## 1. 🏛️ Choose Your Architecture

NHTML adapts to your infrastructure:

| Mode | Usage | Infrastructure | Status |
|:---|:---|:---|:---|
| **Dedicated (Rust)** | **Recommended**. Pure real-time. | VPS / Dedicated (CLI access) | 🟢 Operational |
| **Shared (HTTP)** | Connectivity fallback. | Standard PHP hosting | 🟢 Operational |
| **WASM (Zero-Server)** | 100% Client-side. | GitHub Pages / Static | 🟢 Operational (Zero CDN) |

---

## 2. ⚙️ Quick Installation (Dedicated Mode)

This is the most performant method. The **Rust Gateway** manages WebSockets and supervises your PHP code.

### Step 1: Launch the Gateway
Download the `nhtml` binary and launch it in your project root:
```bash
# Development mode with auto-reload (Default ports: WS=8080, PHP=8000)
./nhtml start --dev

# You can customize ports:
./nhtml start --dev --ws-port 9080 --php-port 9000
```
*The Gateway starts an HTTP server (default on 3000) and the PHP server.*

### Step 2: Project Structure
Place your files in a folder (e.g., `/my-app`):
- `index.nhtml`: Your interface.
- `app.php`: Your business logic.

---

## 3. 🪶 "Zero-Server" Mode (GitHub Pages / Static)

No backend? No problem.
Simply host your files statically (on GitHub Pages, Vercel, or S3).

1. The NHTML Bridge will attempt to reach the server.
2. If no WebSocket server or PHP backend is found, it **automatically switches to WASM mode**.
3. It downloads the PHP WebAssembly virtual machine into the browser.
4. Your `app.php` file is executed locally in RAM at lightning speed.

*You write backend PHP, but it runs in your visitor's browser!*

---

## 4. 🐘 Your First Component

### Client Side (`index.nhtml`)
Identify elements to update with `n-id` and capture clicks with `n-click`.
```html
<h1 n-id="title">Hello</h1>
<button n-click="btn_hello">Click me</button>

<!-- Magic! ZERO <script> tags. The Gateway automatically injects the Bridge JS in development. -->
```

### Server Side (`app.php`)
Use the PHP SDK to send instant binary commands.
```php
<?php
use Nhtml\Nhtml;

if ($event === 'click' && $nodeId === 'btn_hello') {
    Nhtml::patch()
         ->setText('title', "It's magic!")
         ->send();
}
```

---

## 4. 🛰️ Key Concepts for Success

1.  **The Gateway is a Postman**: It doesn't know your business logic; it simply delivers binary packets (NBPS) between PHP and the browser.
2.  **NodeIDs**: Use simple (or numeric) IDs. The server drives the DOM remotely via these IDs.
3.  **Zero Latency (Local Actions)**: For visual effects (hover, scroll), use **Local Actions** (see `SPEC.md`) for immediate execution without a server round-trip.

---

## 📊 Diagnostic Tools
- **DevTools Dashboard**: Visit `http://127.0.0.1:8081` to see your packets in real-time.
- **F12 Console**: Your PHP `error_log()` appear directly in the browser console with the `[NHTML SERVER]` prefix.
