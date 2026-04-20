# Architecture Technique Principale : Nhtml V2

## 1. Objectifs Headless Multi-Cibles
L'objectif de la V2 est de compiler les templates côté serveur ou client, pour produire un couple **(HTML Pur + Manifeste JSON)** ingérant état, réactivité et cycle de vie. Le noyau dur est développé en Rust (`nhtml-core`).

**Trois stratégies de déploiement (Le Pilier Multi-Plateformes) :**
- **Librairie Dynamique (FFI)** : Compilation via C (`.dll` / `.so`). Exploitée par `NhtmlCompiler.php` sur le serveur pour by-passer les lenteurs de démarrage de processus via `FFI`.
- **Exécutable Standalone (CLI)** : Binaire `.exe` / ELF pour servir de fallback serveur (via `exec()`) en cas d'hébergement restrictif.
- **WebAssembly (WASM)** : Transpilation `wasm32-unknown-unknown` empaquetée via `wasm-pack`. Permet la compilation *directement dans le navigateur Google Chrome/Firefox* pour des applications 100% SPA.

## 2. Structure des Modules Nhtml-Core

### 2.1 Parser (`nhtml_parser`)
Analyseur syntaxique fort en Rust utilisant des règles de caractères stricts.
- **Extraction des OpCodes** : Remplace les "évaluations Javascript sauvages (`eval`)" par des commandes sûres (`Increment`, `Set`, `Call`).
- **Support des Rétrocompatibilités** : Gère nativement les anciennes balises `<empty for="...">` en les transpilant à la volée vers une expression conditionnelle if `!(source.length > 0)`.

### 2.2 AST & State (`nhtml_ast`)
- `struct Manifest` : Le conteneur principal de l'état (JSON natif complet grâce à `serde_json`) et de l'arbre comportemental.
- Protège la déclaration des nœuds : `Text`, `If`, `Each`, `Attrs`. 

### 2.3 Bridge / RunTime JS
Le binaire/WASM ne manipulant pas le DOM directement pour des raisons de Sandbox web :
- **Runtime Musclier minimal** : Nhtml v2 injecte une mini librairie JavaScript.
- Son seul rôle : se brancher sur le Manifest JSON (`window._nhtmlAST`), écouter les clics des utilisateurs, envoyer les directives au state Proxy (`window.nhtml`) et hydrater les nœuds ciblés dans le DOM en temps reél.

## 3. Workflow de Distribution
Le projet est déployable en l'état de deux manières distinctes :
* **Distribution Serveur** : Le noyau `nhtml_core.dll` ainsi que `NhtmlCompiler.php` peuvent être poussé par Composer/Packagist ou copiés à la main dans l'écosystème PHP cible comme des extensions silencieuses.
* **Distribution Web (npm)** : Le dossier `pkg/` généré par wasm-pack contient le wrapper Javascript propre servant de module standard (`import init from './pkg/nhtml_core.js'`). Il peut être injecté sur npmjs.org.
