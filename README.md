# ⚛️ Nhtml

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Language-PHP-777bb4?style=flat-square&logo=php" alt="PHP">
  <img src="https://img.shields.io/badge/Language-JavaScript-F7DF1E?style=flat-square&logo=javascript&logoColor=black" alt="JS">
  <img src="https://img.shields.io/badge/Version-2.0-brightgreen?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License">
</p>

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

# 🇫🇷 Version Française

**Nhtml est du HTML avec réactivité intégrée — sans framework, sans outils de build, sans Node.js.**

Nhtml (Native HTML) est un langage de template compilé par un moteur Rust ultra-rapide. Il a été conçu pour apporter l'interactivité moderne à vos applications web tout en supprimant la complexité des écosystèmes JavaScript actuels.

## Pourquoi Nhtml ?

*   🚀 **Performance Native** : Cœur écrit en Rust. Compilation instantanée via FFI.
*   🌍 **Zéro Dépendance** : Pas de `npm install`, pas de `node_modules`, pas de Webpack.
*   💎 **Multi-Cible** : Fonctionne sur Serveur (PHP/C/Rust) et Client (WebAssembly).
*   ⛓️ **Réactivité Légère** : Moins de 2Ko de runtime JS pour l'hydratation du DOM.

## Installation Rapide

Des guides complets sont disponibles dans le dossier [docs/](./docs/).

*   [**Installation PHP (Serveur)**](./docs/INSTALL-PHP.md)
*   [**Installation Serveur Direct (Apache/Nginx)**](./docs/INSTALL-DIRECT.md)
*   [**Installation Navigateur (WASM)**](./docs/INSTALL-WASM.md)

---
© 2026 NemStudio — Powered by simplicity.
