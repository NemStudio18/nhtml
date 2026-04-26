# 🛰️ Référence Technique NHTML (v0.4.0)
**Document de référence pour l'architecture industrialisée.**

---

## 1. Vue d'ensemble du projet
NHTML (Native-HTML) est un framework de développement web "Server-Driven" conçu pour déporter toute la logique métier et l'état de l'application côté serveur tout en offrant une réactivité proche du client natif.

- **Objectif** : Éliminer la complexité du JS frontend en transformant le navigateur en un simple moteur de rendu binaire piloté par un backend (PHP/Rust).
- **Gateway** : Serveur Rust (Tokio, Axum) gérant le transport, la persistance et l'auto-injection.
- **Protocol** : NBPS (Native-HTML Binary Protocol) optimisé avec compression Zstd native.

---

## 2. Architecture Générale
L'architecture repose sur un **Gateway Orchestrateur** agissant comme un pont binaire bidirectionnel.

```mermaid
graph TD
    Browser[Navigateur (bridge.js)] <-->|NBPS Binaire + Zstd| Gateway[Gateway Rust]
    Gateway <-->|JSON/HTTP| PHP[App PHP (SDK)]
    
    Gateway -->|Monitoring| DevTools[Dashboard DevTools 8081]
    Gateway <-->|SQLite| DB_Sessions[nhtml_sessions.db]
    PHP <-->|SQLite| DB_App[app.db]
```

---

## 3. Structure des Répertoires (v0.4.0)
- **`/src/`** : Cœur du système Rust (Gateway & CLI).
- **`/static/`** : Templates du Dashboard DevTools.
- **`/assets/js/`** : Bridge client et polyfills de décompression.
- **`/sdk/php/`** : SDK officiel pour le backend PHP.
- **`/examples/`** : Cas d'usage concrets (Counter, TodoList...).

---

## 4. Protocole NBPS v0.4.0
Le protocole est optimisé pour la bande passante (Zstd) et la fiabilité (Checksums).

- **Header Universel (5 octets)** : `[Type: u8] [Length: u32]`.
- **OpCodes v0.4.0** :
    - `0x01` (HELLO) : Handshake initial et session ID.
    - `0x02` (EVENT) : Interaction client -> serveur (clic, input).
    - `0x03` (PATCH) : Mutations DOM atomiques (setText, setStyle, focus...).
    - `0x05` (SYNC) : Vérification d'intégrité du DOM via checksum.
    - `0x07` (B-TREE) : Snapshot compressé de l'état complet.
    - `0x09` (PING) : Heartbeat de maintien de connexion.
    - `0x7F` (ERROR) : Rapport d'erreur binaire structuré.

---

## 5. Flux d'Injection Automatique
NHTML v0.4.0 simplifie radicalement le déploiement. Le Gateway injecte automatiquement le bridge et les polyfills dans tout fichier `.nhtml` servi.

1. **Requête** : Le client demande `index.nhtml`.
2. **Interception** : Le Gateway lit le fichier, injecte les `<script>` de `bridge.js` et `fzstd.js` avant la balise `</head>`.
3. **Activation** : Le bridge démarre automatiquement la connexion WebSocket vers le Gateway.

---

## 6. Monitoring & Time Travel
Chaque mutation est historisée dans `nhtml_sessions.db`. 
Le Dashboard (port 8081) permet de :
- **Visualiser** les flux en temps réel (Network Monitor).
- **Rejouer** une session étape par étape (Time Travel).
- **Comparer** l'état de deux sessions distinctes.
- **Auditer** la latence exacte des réponses PHP.

---

## 7. Glossaire
- **NID** : Identifiant textuel (string) mappé dynamiquement à un ID binaire u16.
- **Patch Glow** : Feedback visuel signalant une mise à jour réactive.
- **Zero-JS** : Concept où aucun code JavaScript métier n'est écrit par le développeur.
