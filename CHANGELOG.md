# 📜 Changelog NHTML

## v0.4.0 (En cours)
### Ajouté
- **Compression Zstd (Active)** : Les snapshots B-TREE sont désormais compressés par défaut.
- **Monitoring Ratios** : Affichage des gains de compression en temps réel dans le Dashboard DevTools.
- **Benchmark CLI** : Nouvelle commande `nhtml bench` pour comparer les performances NBPS vs HTML.
- **Client Zstd** : Intégration de `fzstd` en local (zéro CDN) dans le polyfill pour la décompression native.
- **Auto-Injection Bridge** : Le Gateway détecte les fichiers `.nhtml` et injecte automatiquement `bridge.js` + `fzstd.js`. Zéro `<script>` à écrire pour le développeur.
- **Ports Configurables** : `nhtml start --ws-port 8080 --php-port 8000`, `nhtml devtools --port 8081`.
- **Pyramide d'Exemples** :
  - `01-counter` — Hello World binaire (existant)
  - `02-todo-list` — APPEND_HTML / REMOVE_NODE sur une liste
  - `03-live-form` — Validation PHP on-input, patches SET_TEXT + SET_CLASS
  - `04-style-lab` — Manipulation CSS en temps réel (border-radius, scale, couleur) via SET_ATTR

## v0.3.1 (Avril 2026)

### ⚠️ Breaking Changes
- **Collision OpCode 0x09** : était `SetStyle` (désormais géré en interne), est devenu **RELOAD** pour les DevTools.
- **Migration B-TREE** : était `0x05`, désormais déplacé en **0x07**.
- **Header Universel** : passage de **3 octets → 5 octets** (`Length u16 → u32`). Format : `[Type:1][Len:4]` (Big-Endian).

### Ajouté
- **0x0B APPEND_HTML** : Ajout incrémental de contenu (Moniteur Réseau & Logs).
- **0x10 LOG** : Relai binaire des logs serveur vers la console F12.
- **NodeVersion (u32)** : Intégration systématique dans les paquets **PATCH** et **B-TREE** pour la résolution de conflits.
- **Supervisor Auto-PHP** : Orchestration automatique du backend par le Gateway Rust.
- **MPSC & Lag Handling** : Refonte de la stabilité WebSocket pour supporter les clients lents sans coupure.

### Corrigé
- UI DevTools : Correction des chevauchements de panneaux sur le Dashboard.
- Bridge JS : Restauration de la verbosité des logs protocolaires (HELLO, SID, IN/OUT).

---

## v0.2.x
- **Baseline initiale** : Support binaire NBPS v0.2.x, Handshake HELLO et mutations DOM basiques.
