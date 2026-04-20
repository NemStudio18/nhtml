# 🚀 NCMS — Nhtml Managed System

![NCMS Logo](https://via.placeholder.com/150)

**NCMS** est un système de gestion de contenu (CMS) ultra-léger, moderne et réactif. Il utilise le moteur de template **Nhtml** pour offrir une expérience utilisateur fluide type Single Page Application (SPA) tout en restant basé sur un backend standard PHP/SQLite.

---

## ✨ Fonctionnalités

- **Moteur Nhtml v2.0** : Architecture "Headless" ultra-performante.
- **Réativité Native** : Pilotage du DOM via un Manifeste JSON (AST) et un Micro-Runtime de 3KB.
## 🚀 État du Projet
- **V2 Prototype (Python)** : 100% Stable. Headless, Deep Binding, Persistance.
- **V2 Core (Rust)** : **OPÉRATIONNEL**. Parser combinatoire haute performance.
- **Suite de Tests** : `kitchen_sink.nhtml` validée.
- **Dashboard Admin** : Une interface d'administration complète pour gérer vos articles.
- **Éditeur Pell Local** : Éditeur WYSIWYG intégré localement pour une stabilité maximale.
- **Backend PHP & SQLite** : Rapide, portable et sans configuration complexe.
- **SEO Ready** : Les métadonnées et titres sont gérés dynamiquement de manière propre.

---

## 🛠️ Installation

### Prérequis
- **PHP 8.x** avec extension SQLite3.
- **Python 3.x** (pour le compilateur Nhtml).
- **Wasm Runtime** (Optionnel, intégré via Micro-Runtime JS par défaut).

### Déploiement rapide
1. Clonez le dépôt.
2. Initialisez la base de données :
   ```bash
   php NCMS/init_db.php
   ```
3. Générez les caches Nhtml :
   ```bash
   python nhtml.py NCMS/templates/admin.nhtml NCMS/public/cache/admin.html
   ```
4. Configurez votre serveur web pour pointer vers `NCMS/public/`.

---

## 📁 Structure du Projet

- `/NCMS/src` : Code source PHP (Controllers, Core, Models).
- `/NCMS/templates` : Fichiers `.nhtml` (structure réactive).
- `/NCMS/public/cache` : Sortie HTML/JS générée par le moteur Nhtml.
- `/NCMS/public/libs` : Bibliothèques locales (Pell).
- `nhtml.py` : Le moteur de transpilation Nhtml.

---

## 📜 Spécifications Nhtml
Le projet NCMS migre vers la spécification **Nhtml v2.0 (Headless)**. Consultez le fichier `NHTML-Specification-v2.0.md` pour plus de détails sur le format du manifeste.

---

## 🛡️ Licence
Ce projet est sous licence MIT. Nhtml est un projet Open Source.
