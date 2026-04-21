# ⚛️ Nhtml

<p align="center">
  <img src="assets/logo.png" width="220" alt="Nhtml Logo">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Engine-Rust-orange?style=for-the-badge&logo=rust" />
  <img src="https://img.shields.io/badge/Runtime-WASM-blue?style=for-the-badge&logo=webassembly" />
  <img src="https://img.shields.io/badge/Hydration-<1.5KB_JS-success?style=for-the-badge" />
</p>

---

## ⚡ The Native HTML Reactivity Engine

**Nhtml is a high-performance rendering engine that brings reactive powers to standard HTML without the weight of a framework.**

Write your UI in pure HTML, compile it with **Rust**, and let Nhtml handle the reactivity. No Node.js, no build steps, just speed.

### 🌟 Why Nhtml?

*   **Zero-JS Logic**: All state mutations and expression evaluations are handled by the Rust engine (WASM).
*   **SSR by Default**: Initial state is pre-rendered by Rust. No "flicker" while waiting for JS.
*   **Ultra-light**: The browser bridge is less than 1.5KB.
*   **Security**: No `eval()` or `new Function()`. Your logic is encapsulated in a secure WASM binary.

---

## 🧠 Simple Syntax, Powerful Engine

```html
<var count=0>

<div class="card">
  <h2>Counter: {count}</h2>

  <button on:click="count++">+</button>
  <button on:click="count--">-</button>

  <if condition="count > 10">
    <p>🔥 Impressive performance</p>
  </if>
</div>
```

---

## ⚔️ Comparison: Nhtml vs The World

| Feature            | Nhtml (Native) | React / Vue | alpine.js |
| ------------------ | -------------- | ----------- | --------- |
| **Logic Engine**   | 🦀 Rust / WASM | 🟨 JS       | 🟨 JS     |
| **Initial Load**   | ✅ SSR Native  | ⚠️ Client   | ⚠️ Client |
| **Security**       | ✅ High (WASM) | ⚠️ Mixed    | ❌ Unsafe |
| **Learning Curve** | ✅ Zero        | ❌ High     | ✅ Low    |
| **Bundle Size**    | ✅ < 2KB       | ❌ 40KB+    | ✅ 10KB   |

---

## 🚀 Key Features

*   **Native SSR**: Pre-render everything on the server (PHP/Rust) with the same engine as the browser.
*   **Event Delegation**: High-performance event handling driven by Rust.
*   **Property Bindings**: Bind attributes, styles, and classes reactively.
*   **Array Support**: Manage lists with native `.push()` and indexing.

---

## 🛠️ Usage

### 1. In the Browser (WASM)
Just drop the WASM package and you're ready. See the [Playground](index.html) for a live demo.

### 2. Server-side (PHP/Rust)
Compile Nhtml directly into your server-rendered templates for instant initial display.

---

## 📖 Documentation & Guides

Consult the [`docs/`](./docs/) directory:
*   [WASM Browser Integration](./docs/INSTALL-WASM.md)
*   [PHP Server Integration](./docs/INSTALL-PHP.md)
*   [Architecture Deep-Dive](./docs/ARCHITECTURE.md)

---

## 🎯 Philosophy
Nhtml is built on the belief that **HTML should be enough**. We don't need a massive toolchain to make a button counter. We need a faster engine for the tools we already have.

---

© 2026 NemStudio — Built for simplicity and speed.
