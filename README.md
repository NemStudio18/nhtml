# 🛰️ NHTML — The Server-Driven DOM Protocol

<p align="center">
  <img src="assets/logo.png" width="250" alt="NHTML Logo">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Version-v0.4.0-ff007f?style=flat-square" />
  <img src="https://img.shields.io/badge/Gateway-Rust-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/Backend-PHP_8.x-777bb4?style=flat-square&logo=php" />
  <img src="https://img.shields.io/badge/Transport-Binary_NBPS-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/Business_JS-Zero-success?style=flat-square" />
</p>

[English](#-english-version) | [Français](#-version-française)

---

## 🇬🇧 English Version

### ⚡ Reactive web. Without writing JavaScript.

**NHTML is a Server-Driven UI framework.** You write augmented HTML and PHP. The Rust Gateway handles the rest — binary transport, atomic DOM mutations, and native real-time.

Zero React. Zero Vue. Zero build step. Zero business JS to write.

---

### 🧠 What it looks like

**Interface side (`index.nhtml`)** — HTML with super-powers:

```html
<h1 n-id="title">Hello</h1>

<p>
  You clicked
  <strong n-id="counter">0</strong> times.
</p>

<button n-click="page.increment">Click here</button>

<div n-id="message" n-live></div>
```

**Server side (`app.php`)** — Pure PHP:

```php
function page_increment(NhtmlEvent $event): array
{
    $count = ++$_SESSION['clicks'];

    return [
        Patch::setText('counter', (string)$count),
        Patch::setText('message', $count === 10 ? '🎉 Tenth click!' : ''),
    ];
}
```

👉 No JSON to write. No fetch(). No client-side state to manage.

---

### 🖼️ Showcase in Action

![NHTML Showcase Dashboard](assets/showcase_preview.png)
*The Premium Showcase MVC demonstrating real-time data binding, inventory management, and multi-view navigation.*

---

### 🛠️ Built-in DevTools

NHTML includes a complete control station accessible locally at `http://127.0.0.1:8081`:

![NHTML DevTools Preview](assets/devtools_preview.png)

- **Network Monitor** — every NBPS packet in real-time.
- **Time Travel** — replay any session action by action.
- **Node Inspector** — binary state of every DOM node.
- **State Diff Viewer** — visualize mutations before/after.

---

### 🏛️ How it works

```
Your .nhtml + app.php
        │
        ▼
[ Rust Gateway ]  ←──────────────────────────────┐
        │                                         │
        │  WebSocket (Binary NBPS Protocol)       │
        │                                         │
        ▼                                         │
[ Browser ]                                 [ PHP Backend ]
  bridge.js (~25KB)   EVENT (click) ────────▶  Business Logic
  Applies PATCHES     PATCH (mutations) ◀────  Returns ops
```

---

### 🚀 Quick Start

```bash
# 1. Start the gateway
./nhtml start --dev

# 2. Visit http://localhost:8080
```

[Read full English Documentation](./docs/en/SPEC.md)

---

## 🇫🇷 Version Française

### ⚡ Le web réactif. Sans écrire de JavaScript.

**NHTML est un framework Server-Driven UI.** Vous écrivez du HTML augmenté et du PHP. Le Gateway Rust s'occupe du reste — transport binaire, mutations DOM atomiques, temps réel natif.

Zéro React. Zéro Vue. Zéro build step. Zéro JS métier à écrire.

---

### 🏛️ Comment ça marche

```
Votre .nhtml + app.php
        │
        ▼
[ Gateway Rust ]  ←──────────────────────────────┐
        │                                         │
        │  WebSocket (Protocole NBPS Binaire)     │
        │                                         │
        ▼                                         │
[ Navigateur ]                              [ PHP Backend ]
  bridge.js (~25KB)   EVENT (clic) ────────▶  Logique métier
  Applique les PATCH  PATCH (mutations) ◀────  Renvoie les ops
```

---

### ⚖️ Licence

- **NHTML** (Gateway, SDK, Bridge) — [AGPL v3](./LICENSE)

---

<p align="center">
  © 2026 NemStudio18 — Built for simplicity. Powered by Rust.
</p>
