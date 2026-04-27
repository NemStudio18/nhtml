# ⚙️ Installation & Setup

[English](#-english) | [Français](#-français)

---

## 🇬🇧 English

NHTML can be installed and deployed in 3 different ways depending on your hosting provider.

### 1. Dedicated Mode (High Performance)
Best for VPS or Dedicated servers where you can run the `nhtml` binary.
- Download the version for your OS from [Releases](https://github.com/NemStudio18/nhtml/releases).
- Run: `./nhtml start --dev`

### 2. Shared Mode (Standard Hosting)
Best for OVH, o2switch, etc. where you have FTP and PHP but no binary access.
- Upload your `.nhtml`, `.php` files and the `assets/js` folder via FTP.
- NHTML will automatically use HTTP fallback for reactive updates.

### 3. Zero-Server Mode (Static Hosting)
Best for GitHub Pages, Netlify, or S3.
- Push your files to your static repository.
- NHTML will automatically run your PHP logic in the browser using **WebAssembly**.

### 📚 Detailed Guides
- [Getting Started Guide](./docs/en/GETTING_STARTED.md)
- [Deployment Guide (Nginx/Apache)](./docs/en/DEPLOYMENT.md)
- [Architecture Overview](./docs/en/ARCHITECTURE.md)

---

## 🇫🇷 Français

NHTML peut être installé et déployé de 3 manières différentes selon votre hébergeur.

### 1. Mode Dédié (Haute Performance)
Idéal pour les VPS ou serveurs dédiés où vous pouvez lancer le binaire `nhtml`.
- Téléchargez la version correspondant à votre OS depuis les [Releases](https://github.com/NemStudio18/nhtml/releases).
- Lancez : `./nhtml start --dev`

### 2. Mode Mutualisé (Hébergement Classique)
Idéal pour OVH, o2switch, etc. où vous avez un accès FTP et PHP mais pas aux binaires.
- Envoyez vos fichiers `.nhtml`, `.php` et le dossier `assets/js` par FTP.
- NHTML utilisera automatiquement le fallback HTTP pour les mises à jour réactives.

### 3. Mode Zéro-Serveur (Hébergement Statique)
Idéal pour GitHub Pages, Netlify, ou S3.
- Poussez simplement vos fichiers sur votre dépôt statique.
- NHTML exécutera automatiquement votre logique PHP dans le navigateur via **WebAssembly**.

### 📚 Guides Détaillés
- [Guide de Démarrage Rapide](./docs/fr/GUIDE_DEMARRAGE.md)
- [Guide de Déploiement (Nginx/Apache)](./docs/fr/DEPLOIEMENT.md)
- [Spécification du Protocole](./docs/fr/SPEC.md)

---
© 2026 NemStudio — AGPL-3.0 License
