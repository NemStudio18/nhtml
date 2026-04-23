# 🛠️ Roadmap : Écosystème d'Outils NHTML (v0.2.2+)

Ce document définit la stratégie de développement des outils entourant le protocole NHTML pour faciliter son adoption, son débogage et l'expérience développeur (DX).

---

## 📅 Phase 1 : Fondations DevTools (1-2 semaines)
*Priorité : Rendre le flux binaire transparent et inspectable.*

### 🔍 1.1 Décodeur de Messages Binaires (Très Haute)
- [x] **Core Decoder** : Module Rust/JS capable de traduire tout paquet NBPS en JSON structuré.
- [x] **CLI Inspect** : Commande `nhtml inspect` pour dumper un flux binaire en temps réel.
- [ ] **Web Inspector UI** : Interface intégrée au Polyfill pour visualiser les échanges dans le navigateur.

### 🌳 1.2 Visualiseur d'État du DOM (Très Haute)
- [x] **Snapshot Explorer (CLI)** : Commande `db-dump` améliorée pour explorer l'état SQLite.
- [ ] **Diff Highlighter** : Coloration des zones du DOM impactées par le dernier `PATCH`.

---

## 📅 Phase 2- [x] Industrialisation du Moteur de Replay (v0.2.3)
- [x] Résolution des conflits de ports (Gateway: 8080, Replay: 8081)
- [x] Portabilité totale via embedding des ressources (include_str!)
- [x] Détection dynamique de la base de données SQLite
- [x] **Modular Refactor** : Structure du CLI refactorisée en modules (`cli.rs`).
- [ ] **`nhtml generate`** : Création automatique de composants et de messages types.
- [ ] **Dev Server** : Amélioration du superviseur avec support auto-certificat SSL.

### ⏱️ 2.2 Enregistreur de Sessions / Time Travel (Très Haute)
- [x] **Event Logging** : Archivage systématique dans SQLite opérationnel.
- [ ] **Persistence Replay** : Utilisation du log SQLite pour rejouer une session utilisateur bit-à-bit.
- [ ] **Seek Bar** : Curseur temporel pour naviguer entre les états passés du DOM.

---

## 📅 Phase 3 : SDKs & Intégration IDE (1-2 semaines)
*Priorité : Intégration profonde dans le workflow des développeurs.*

### 📦 3.1 SDKs Officiels (Très Haute)
- [x] **PHP SDK v0.2.2** : Bibliothèque binaire NBPS validée.
- [ ] **JS SDK** : Client standalone pour intégration hors Polyfill standard.

### 🔌 3.2 Plugins IDE (Haute)
- [ ] **VSCode Extension** : Surlignage `.nhtml`, autocomplétion des OpCodes et intégration de l'inspecteur.

---

## 📅 Phase 4 : Diagnostic & Performance (1 semaine)
*Priorité : Optimisation et robustesse.*

### 📊 4.1 Moniteur de Performances
- [ ] **Latency Tracker** : Mesure du RTT click-to-patch.
- [ ] **Bandwidth Stats** : Comparaison en temps réel Binaire vs JSON.

---

## 🏗️ État Actuel du Développement
- **Phase 1** : 🟢 **70%** (Décodeur et CLI Inspect OK)
- **Phase 2** : 🟡 **40%** (Logging OK, Refactor CLI OK)
- **Phase 3** : 🟡 **20%** (SDK PHP Binaire OK)
- **Phase 4** : ⚪ En attente
