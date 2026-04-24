# 📂 Architecture des Répertoires et Fichiers de NHTML

Ce document détaille la structure de fichiers du projet **NHTML** suite au "Grand Split". Il sert de point de référence pour comprendre à quoi sert chaque dossier et chaque fichier majeur du système.

---

## 🌳 Arborescence Générale

```text
nhtml-v1-repo/
├── nhtml-gateway/           # Le Cœur du Réacteur (Rust) - Serveur et Compilateur
├── nhtml-core/              # L'Écosystème Client et Backend (SDK PHP, Assets JS)
├── nhtml-cloud/             # (Futur) Déploiement et infrastructure SaaS NHTML
├── docs/                    # Documentation technique et spécifications
├── README.md                # Le manifeste et guide de démarrage rapide
├── STRUCTURE.md             # Ce document
└── .gitignore               # Exclusions de suivi Git
```

---

## ⚙️ `nhtml-gateway/` — Le Gateway Rust (Serveur & Compilateur)
Ce dossier contient le moteur principal écrit en Rust (licence AGPL). C'est un binaire unique, ultra-rapide et totalement autonome, responsable de compiler le `.nhtml`, de démarrer les sessions et de piloter les websockets.

- **`Cargo.toml`** : Le fichier des dépendances Rust (Axum, Tokio, etc.).
- **`src/main.rs`** : Le point d'entrée. Démarre le serveur HTTP, inclut directement les ressources WASM/JS dans le binaire (via macros `include_bytes!`), et expose les endpoints.
- **`src/cli.rs`** : Contient la boucle d'événements réseau (WebSocket). Gère l'envoi de la séquence `HELLO` et le traitement continu des requêtes `EVENT`/`PATCH` en lien avec le backend PHP.
- **`src/watcher.rs`** : Le module qui surveille les fichiers sources (Live Reloading). Si un fichier PHP ou `.nhtml` est modifié, il recompile et pousse un rafraîchissement global (`RELOAD`) au client.
- **`src/session.rs`** : Gère l'état d'une session utilisateur active (NodeMap, historique, versions de nœuds pour la cohérence).
- **`src/decoder.rs`** : Décodeur binaire du protocole propriétaire NHTML (B-TREE et EVENT).
- **`src/proto.rs`** : Contient les constantes d'opération du protocole NBPS (ex: `OP_SET_TEXT = 0x01`) et les constructeurs de trames binaires.
- **`src/compiler/`** : Le sous-module responsable de la transformation du HTML brut en arbre binaire B-TREE.
  - `mod.rs` : Le parseur syntaxique qui repère les attributs de protocole (ex: `n-id`, `n-click`) et génère les actions de Binding.
  - `btree_builder.rs` : Le sérialiseur qui convertit l'arbre syntaxique en tableau d'octets optimisé (byte-array) prêt à être compressé.

---

## 🧩 `nhtml-core/` — L'Écosystème Client & Développeur
Ce répertoire contient le nécessaire pour construire des applications NHTML (le SDK côté serveur et les bibliothèques JS côté client).

### 1. Le sous-dossier `examples/` (Assets & JS Polyfills)
Héberge les exemples d'implémentation officiels ainsi que les bibliothèques frontend du framework.
- **`assets/js/bridge.js`** : **Le Moteur Front-End (Client)**. Ce fichier JavaScript s'exécute dans le navigateur. Il intercepte les clics et les inputs, communique avec le Gateway, décompresse le B-TREE, et applique les patchs DOM de façon chirurgicale. Il intègre le fallback automatique vers PHP-WASM.
- **`assets/js/php-wasm/`** : La machine virtuelle PHP compilée en WebAssembly, permettant de faire tourner le code métier PHP directement dans le navigateur (mode "Zero-Server").
  - `php-web.mjs.wasm` : Le binaire massif de la machine virtuelle (empaqueté dans Rust).
  - `PhpWeb.mjs` : Le script d'interface JS qui amorce le moteur WASM.
- **`assets/js/fzstd.min.js`** : Librairie de décompression Zstd pour réduire drastiquement la taille de l'arbre B-TREE lors de l'initialisation réseau.

### 2. Le sous-dossier `sdk/php/` (Le SDK Backend PHP)
C'est l'interface d'écriture métier offerte au développeur.
- **`src/Nhtml.php`** : La classe principale exposée (`Nhtml::setText()`, `Nhtml::addClass()`, etc.).
- **`src/Gateway.php`** : La couche de communication bas niveau. Reçoit le contexte d'exécution JSON via `stdin` ou `POST` et renvoie les patchs DOM.
- **`src/Event.php`** : L'objet représentant l'événement déclenché par l'utilisateur (clic, texte tapé).
- **`src/Patch.php`** : La représentation d'une mutation unitaire du DOM (ex: un changement de couleur).
- **`src/Protocol/Encoder.php`** : Le traducteur qui prend les instances de `Patch` PHP et les formate en tableau binaire `0x03 PATCH` compréhensible par le Gateway Rust.

---

## ☁️ `nhtml-cloud/` — Déploiement et Infrastructure
Dossier actuellement vide, il sera l'hôte des scripts d'orchestration (Docker, Kubernetes, Terraform) pour la future plateforme mutualisée et les configurations Nginx/Apache.

---

## 📚 Les Documents de Référence (Dossier `/docs`)
L'ensemble de ces documents maintient la cohérence de NHTML, qui se définit avant tout comme un protocole de transport d'UI (NBPS).
- **`docs/SPEC.md`** : Le livre blanc technique, définissant tous les OpCodes et la topologie des trames binaires. La source de vérité absolue.
- **`docs/ARCHITECTURE.md`** : L'explication visuelle des flux de données, de la pression d'un bouton jusqu'à la mise à jour asynchrone du DOM.
- **`docs/GUIDE_DEMARRAGE.md`** : Tutoriel d'initiation aux trois modes de déploiement (Local, Serveur, WASM).
- **`docs/DEPLOIEMENT.md`** : Recommandations pour l'hébergement en production.
- **`docs/CHANGELOG.md`** : Historique des "breaking changes" et des évolutions du protocole.
