# 📂 Architecture des Répertoires et Fichiers de NHTML

Ce document détaille la structure de fichiers du projet public **NHTML Gateway** (licence AGPL). 
Afin de maximiser la simplicité des contributions, le moteur Rust, les SDKs et les bibliothèques JS sont réunis dans ce même dépôt.

---

## 🌳 Arborescence Générale

```text
nhtml-gateway/               ← (Le repo actuel)
├── static/                  # Templates statiques (DevTools, Dashboard)
├── src/                     # Moteur Rust (Serveur HTTP / WS, Monitoring)
├── sdk/                     # SDKs Backend officiels
│   ├── php/                 # SDK PHP v0.4.0 (Nhtml.php, Protocol/)
├── assets/js/               # Bridge Client et Polyfills (zstd)
├── examples/                # Démos (Counter, TodoList, StyleLab)
├── docs/                    # SPEC.md, ARCHITECTURE.md
├── Cargo.toml               # Dépendances Rust
└── README.md                # Guide de démarrage rapide
```

---

## ⚙️ `src/` — Le Gateway Rust (Cœur de l'Écosystème)
- **`main.rs`** : Serveur HTTP/WS, Injection automatique du Bridge, Relais PHP.
- **`session.rs`** : Gestionnaire de sessions persistant via SQLite.
- **`proto.rs`** : Définition binaire du protocole NBPS v0.4.0.
- **`cli.rs`** : CLI du projet et Serveur DevTools (Dashboard de monitoring).
- **`decoder.rs`** : Décodeur binaire pour l'inspection des messages.

---

## 📊 `static/` — Ressources Dashboard
- **`devtools.nhtml`** : L'interface du dashboard de monitoring (v0.4.0).

---

## 🧩 `sdk/` — L'Écosystème Backend
- **`sdk/php/src/Nhtml.php`** : Interface unifiée (setText, setStyle, scrollTo, etc.).

---

## 🌐 `assets/js/` — Le Moteur Frontend
- **`bridge.js`** : Interpréteur binaire ultra-léger (DOM Mutator).
- **`fzstd.min.js`** : Polyfill de décompression pour le support Zstd.

---

## 📚 `docs/` — La Documentation Formelle
- **`SPEC.md`** : Spécification binaire complète (La source de vérité).
- **`ARCHITECTURE.md`** : Schémas des flux de données.
