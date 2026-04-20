# Architecture Technique : nhtml-core (Rust)

## 1. Objectifs
L'objectif de `nhtml-core` est de porter la logique de compilation et de runtime Nhtml v2 (Headless) dans un langage système performant.
- **Multicible** : Un seul code source pour générer une bibliothèque dynamique (`.so` / `.dll`) et un module WebAssembly (`.wasm`).
- **Performance** : Remplacer le moteur de recherche par regex (Python) par un véritable analyseur syntaxique (Parser).
- **Stabilité** : Utilisation du typage fort de Rust pour garantir l'intégrité de l'AST.

## 2. Structure des Modules (Projetés)

### 2.1 Parser (`nhtml_parser`)
Implémentation terminée en utilisant la bibliothèque `nom`.
- **Mécanisme** : Le parser utilise une descente récursive pour capturer les blocs imbriqués (`<if>`, `<each>`).
- **Détection** : Scan automatique des balises HTML pour identifier les attributs réactifs (`on:`, `bind:`).

### 2.2 AST & State (`nhtml_ast`)
- `struct Manifest` : Le conteneur principal de l'état et des nœuds réactifs.
- `enum Node` : `Text`, `If`, `Each`, `Attrs`. 
- **Sécurisation** : Utilisation de `serde` pour une sérialisation JSON compatible avec le runtime JS/Vasm.

### 2.3 Runtime & Renderer (`nhtml_runtime`)
- **Mode Serveur** : Générateur de String (SSR) ultra-rapide.
- **Mode Client (Wasm)** : Utilisation de `web-sys` pour manipuler le DOM du navigateur via les IDs `n_x`.

## 3. Roadmap de Développement
1.  **Skeleton** : Initialisation avec `cargo` et définition des structures `ast`.
2.  **Parser POC** : Un parser capable de lire les balises `<var>` et de générer le JSON.
3.  **Wasm Bridge** : Première preuve de concept d'hydratation du DOM depuis Rust.
