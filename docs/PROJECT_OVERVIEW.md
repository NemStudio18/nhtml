# 🚀 NHTML Project Overview (v0.2.2)

## 🎯 La Vision
NHTML est un framework "Native-First" conçu pour éliminer la complexité du JavaScript côté client en déléguant toute la logique métier au serveur (PHP/Rust) tout en conservant une réactivité de type application native. 

**Le concept clé :** Le serveur ne renvoie pas de HTML, il renvoie des **instructions de modification binaires atomiques (NBPS Patches)** sur un arbre synchronisé.

---

## 🏗️ Architecture Industrielle (v0.2.2)

Le système repose sur trois piliers :

1.  **Le Gateway (Rust)** : Le superviseur de session.
    - Gère les flux **NBPS (Native HTML Binary Protocol Specification)**.
    - **Persistence** : Archivage en temps réel dans SQLite (`nhtml_sessions.db`).
    - **Sécurité P3** : Validation systématique de l'intégrité des événements.
2.  **Le Backend (SDK PHP/Rust)** : Le cerveau applicatif.
    - Reçoit les interactions via le Gateway.
    - Utilise le **SDK Binaire** pour piloter le DOM avec une latence réseau minimale.
3.  **Le Polyfill (WASM)** : Le terminal léger.
    - Applique les mutations binaires à une vitesse native.
    - Zéro logique métier, 100% rendu et capture d'événements.

---

## 📊 État Actuel du Projet (Avril 2026)

### ✅ Ce qui fonctionne (Production-Ready)
- **Protocole Binaire NBPS v0.2.2** : Communications optimisées (10x plus légères que le JSON).
- **Triple-Path Resync** : Gestion intelligente de la synchronisation (Fast, Delta, Full).
- **Zéro-Configuration** : Environnement prêt à l'emploi avec `nhtml start`.
- **Persistance SQLite** : Auditabilité complète de chaque interaction utilisateur.

### 🔄 Ce qui est en cours (Focus Écosystème)
- **Record/Replay** : Développement de l'outil pour "rejouer" les sessions bit-à-bit.
- **SDKs Multi-Langages** : Finalisation du SDK PHP Pro et début du SDK Rust.
- **NHTML CLI** : Outils de diagnostic avancés (`inspect`, `db-dump`).

### 🔭 Horizon (Le futur)
- **Embedded PHP** : Intégrer PHP directement dans le binaire Rust.
- **Native Browser Support** : Standardisation du protocole NBPS.

---

## 📂 Organisation des fichiers
- `/gateway/src` : Le moteur Rust industriel.
- `/sdk/php` : Le SDK professionnel binaire.
- `/docs` : Spécifications techniques et Roadmap.

**Status Global :** **v0.2.2 INDUSTRIELLE**. Le cœur est scellé. On construit maintenant les outils de l'écosystème.
