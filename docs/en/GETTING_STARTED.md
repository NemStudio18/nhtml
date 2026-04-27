# 🚀 NHTML Getting Started Guide (v0.6.0)
**Welcome to the Web without JavaScript. Native Performance, Server Logic.**

This guide will walk you through installation to your first reactive interaction.

---

## 1. 🏛️ Choose Your Architecture

NHTML adapts to your infrastructure:

| Mode | Usage | Infrastructure | Status |
|:---|:---|:---|:---|
| **Dedicated (FastCGI)** | **Recommended**. High performance. | VPS / Dedicated (PHP-FPM) | 🟢 Operational |
| **Dedicated (Rust CGI)** | Development simplicity. | VPS / Dedicated (PHP CLI) | 🟢 Operational |
| **WASM (Zero-Server)** | 100% Client-side. | GitHub Pages / Static | 🟢 Operational |

---

## 2. ⚙️ Quick Installation (Dedicated Mode)

This is the most performant method. The **Rust Gateway** manages WebSockets and supervises your PHP code.

### Step 1: Launch the Gateway
Download the `nhtml` binary and launch it in your project root.

**Standard Mode (CGI):**
```bash
nhtml start --dev
```

**High Performance Mode (FastCGI):**
If you have a PHP-FPM pool running (e.g., on port 9000):
```bash
nhtml start --dev --fpm 127.0.0.1:9000
```

### Step 2: Project Structure
Place your files in a folder (e.g., `/my-app`):
- `index.nhtml`: Your interface.
- `app.php`: Your business logic.

---

## 3. 🛡️ Built-in Security (v0.5.0+)

NHTML v0.5.0 introduces **Industrial Security** by default:
- **HMAC-SHA256**: All client events are cryptographically signed. The Gateway rejects any non-authentic frames.
- **Sequence IDs**: Every action is numbered. Replay attacks are natively blocked.

---

## 4. 📡 Real-Time Collaboration (v0.6.0+)

You can now synchronize multiple users instantly.
In your `app.php`, you can request the Gateway to broadcast a patch:
```php
// Send the message to ALL other users in the session
Nhtml::broadcast('others')
     ->setText('notif', "A new user has joined!")
     ->send();
```

---

## 5. 🐘 Your First Component

### Client Side (`index.nhtml`)
```html
<h1 n-id="title">Hello</h1>
<button n-click="btn_hello">Click me</button>
```

### Server Side (`app.php`)
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

## 📊 Diagnostic Tools
- **DevTools Dashboard**: Visit `http://127.0.0.1:8081` to see your signed packets in real-time.
- **F12 Console**: Your PHP `error_log()` appear directly in the browser console.
