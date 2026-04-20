# ⚛️ Nhtml

**Nhtml is HTML with built-in reactivity — no framework, no build tools, no Node.js.**

Nhtml (Native HTML) is a template language compiled by an ultra-fast Rust engine, designed to bring modern interactivity to your web applications without the complexity of today's JavaScript ecosystems.

```html
<!-- Your .nhtml component -->
<var count=0>

<div class="card">
    <h2>Counter: {count}</h2>
    
    <button on:click="count++">Increment</button>
    <button on:click="count--">Decrement</button>
    
    <if condition="count > 10">
        <p>🔥 Impressive score!</p>
    </if>
</div>
```

## Why Nhtml?

*   🚀 **Native Performance**: Core written in Rust. Instant compilation via FFI.
*   🌍 **Zero Dependencies**: No `npm install`, no `node_modules`, no Webpack.
*   💎 **Multi-Target**: Works on Server (PHP/C/Rust) and Client (WebAssembly).
*   ⛓️ **Sustainable Reactivity**: Under 2KB JS runtime to hydrate the DOM.

## Quick Installation

Full guides are available in the [docs/](./docs/) directory.

*   [**PHP Installation (Server)**](./docs/INSTALL-PHP.md)
*   [**Direct Server Installation (Apache/Nginx)**](./docs/INSTALL-DIRECT.md)
*   [**Browser Installation (WASM)**](./docs/INSTALL-WASM.md)

---

### 🇫🇷 Version Française

**Nhtml est HTML avec la réactivité intégrée — sans framework, sans build tool, sans Node.js.**

Nhtml est un langage de template compilé par un moteur Rust ultra-rapide, conçu pour apporter de l'interactivité moderne à vos applications Web sans la complexité des écosystèmes JS actuels.

Consultez le guide d'installation complet [INSTALL.md](./INSTALL.md).

---
© 2026 NemStudio — Powered by simplicity.
