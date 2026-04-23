# 📜 NHTML MASTER SPECIFICATION (v0.2.2)
## "The Industrial Grade Architecture"

---

### 1. Vision Stratégique
NHTML élimine la duplication de logique entre le client et le serveur. Au lieu de synchroniser manuellement un état JS et une base de données, NHTML centralise toute l'intelligence côté serveur.
Le navigateur redevient un **terminal léger**, piloté par un flux binaire haute performance (**NBPS**).

**La Promesse :** 
- **Single Source of Truth** : 100% de la logique réside dans ton SDK (PHP, Go, Rust...).
- **Zéro JS Développeur** : Pas de frameworks front-end à apprendre ou à maintenir.
- **Latence Perçue 0ms** : Grâce aux Local Actions natives (Hover, Scroll, Mouse).
- **Sécurité Native (P3)** : Validation systématique de chaque interaction contre l'état serveur.

---

### 2. L'Architecture "Triple-Path"
NHTML v0.2.2 introduit une gestion de synchronisation à trois niveaux :

1.  **Fast Path (Mutation)** : Envoi de `PATCH` binaires ultra-légers pour les changements d'état.
2.  **Delta Path (Resync)** : Synchronisation automatique des versions de nœuds en cas de décalage mineur.
3.  **Full Path (B-TREE)** : Reconstruction complète de l'interface via un arbre binaire compressé Zstd en cas de déconnexion majeure.

---

### 3. Les Composants du Système

#### A. Le Gateway (The Supervisor - Rust)
Binaire central assurant le routage binaire, la persistence SQLite et la supervision des processus back-end (PHP-CGI).

#### B. Le Renderer (The Terminal - WASM/Natif)
Micro-runtime (WASM 5 Ko) ou moteur natif exécutant les ordres NBPS et gérant les Local Actions sans latence réseau.

#### C. Le SDK (The Engine - PHP, Rust...)
Bibliothèques professionnelles permettant d'encoder les messages NBPS v0.2.2 et de piloter l'interface via une API Fluent.

---

### 4. Le Protocole Binaire NBPS v0.2.2

| OpCode | Paquet | Fonction |
| :--- | :--- | :--- |
| **0x01** | **HELLO** | Handshake binaire et authentification session (UUID v4). |
| **0x02** | **EVENT** | Interaction utilisateur sécurisée (NodeID). |
| **0x03** | **PATCH** | Mutation DOM granulaire (SetText, AddClass, etc.). |
| **0x04** | **SYNC** | Alignement de la version globale de l'état. |
| **0x05** | **BTREE** | Snapshot complet du DOM compressé Zstd + Checksum CRC32. |
| **0x06** | **PING** | Heartbeat et mesure de latence. |

---

### 5. Observabilité & Diagnostic (NHTML CLI)
Le Gateway v0.2.2 intègre des outils de diagnostic de premier ordre :
- **`inspect`** : Traduction temps-réel du binaire en JSON structuré.
- **`db-dump`** : Inspection de la base de données de session SQLite.
- **`validate`** : Vérification de la conformité des flux binaires.

---

### 6. Roadmap d'Évolution

#### Phase 1 : Industrialisation (TERMINÉE)
- Passage au protocole binaire complet v0.2.2.
- Persistence SQLite et Sécurité P3.
- Refactorisation modulaire du Gateway Rust.

#### Phase 2 : Écosystème Tools (EN COURS)
- Finalisation des SDKs officiels (PHP v1.0, JS).
- Développement du moteur de **Replay & Time Travel**.
- Plugins IDE (VSCode Extension).

#### Phase 3 : Native First
- Distribution du fork Chromium NHTML.
- Standardisation W3C.

---
**NHTML : One binary. Any language. Native speed.**
