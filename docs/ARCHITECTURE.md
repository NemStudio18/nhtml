# 🏗️ Nhtml Architecture

Nhtml is designed as a **Universal Rendering Engine** written in Rust, compiled to WASM for the browser, and usable as a native library for server-side environments.

## 1. Core Principles

### Isomorphic Rendering
The same Rust engine that renders the initial HTML on the server (SSR) is used in the browser to handle live updates. This ensures absolute consistency between what the user sees first and how the app behaves next.

### Logic Encapsulation
Unlike traditional reactive frameworks (React, Alpine, Vue) that execute logic in the browser's JavaScript environment, Nhtml executes all logic inside a **secure WebAssembly (WASM) binary**.

## 2. Data Flow

1.  **Compilation Phase**:
    -   The Nhtml source code is parsed into an AST.
    -   Initial state is evaluated in Rust.
    -   HTML is generated with pre-rendered values (SSR).
    -   A **Manifest** is generated, mapping IDs to reactive logic (Expressions, If-conditions, Loops).

2.  **Hydration Phase**:
    -   The browser receives the full HTML.
    -   A tiny (<1.5KB) JS bridge loads the WASM engine.
    -   The bridge attaches **Global Event Listeners** to the root (Event Delegation).

3.  **Update Phase**:
    -   An event (e.g. `click`) occurs.
    -   The JS bridge captures the event and sends the current `state` + the defined `OpCode` to the Rust engine (`dispatch_wasm`).
    -   Rust mutates the state and calculates **only the necessary DOM updates**.
    -   The JS bridge applies these updates (e.g. `textContent`, `style`, `innerHTML` for loops).

## 3. Technical Stack

-   **Core**: Rust 1.75+
-   **AST/Serialization**: `serde`, `serde_json`
-   **Parsing**: `nom` (Fast parser combinators)
-   **WASM**: `wasm-pack`, `wasm-bindgen`
-   **Frontend**: Vanilla JS (The Bridge)

## 4. Performance Benefits

-   **Zero-Eval**: Moving logic to WASM removes the need for `eval()` or `new Function()`, making it faster and more secure.
-   **Minimal Main Thread Usage**: Complex calculations happen in WASM, leaving the JS main thread free for UI rendering.
-   **Bundless Interactivity**: No need for Webpack or Vite. Just drop the Nhtml engine and write HTML.

---

© 2026 NemStudio — Advanced Agentic Coding Project.

