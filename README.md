# 🚀 NCMS — Nhtml Managed System

![NCMS Logo](https://via.placeholder.com/150)

**NCMS** est un système de gestion de contenu (CMS) ultra-léger, moderne et réactif. Il utilise le moteur de template **Nhtml** pour offrir une expérience utilisateur fluide type Single Page Application (SPA) tout en restant basé sur un backend standard PHP/SQLite.

---

## ✨ Fonctionnalités

- **Moteur Nhtml v0.3** : Réactivité native sans framework (React/Vue/etc.).
- **Dashboard Admin** : Une interface d'administration complète pour gérer vos articles.
- **Éditeur Pell Local** : Éditeur WYSIWYG intégré localement pour une stabilité maximale.
- **Backend PHP & SQLite** : Rapide, portable et sans configuration complexe.
- **Routing Intelligent** : Gestion fluide des vues (Liste, Édition, Nouvel article).
- **SEO Ready** : Les métadonnées et titres sont gérés dynamiquement de manière propre.

---

## 🛠️ Installation

### Prérequis
- **PHP 8.x** avec extension SQLite3.
- **Python 3.x** (pour le moteur Nhtml).
- **Serveur Web** (Nginx recommandé, compatible Apache).

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
Le projet NCMS est basé sur la spécification **Nhtml v0.3**. Consultez le fichier `NHTML-Specification-v0.3.md` pour plus de détails sur la syntaxe.

---

## 🛡️ Licence
Ce projet est sous licence MIT. Nhtml est un projet Open Source.
