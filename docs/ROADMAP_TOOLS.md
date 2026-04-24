# 🛠️ Roadmap : Écosystème d'Outils NHTML (v0.3.0)

Ce document définit la stratégie de développement des outils entourant le protocole NHTML pour faciliter son adoption, son débogage et l'expérience développeur (DX).

---

## 📅 Phase 1 : Fondations DevTools (TERMINÉE 🟢)
- [x] **Core Decoder** : Module Rust/JS binaire NBPS v0.3.1.
- [x] **CLI Inspect** : Commande `nhtml inspect` fonctionnelle.
- [x] **NHTML Glow** : Diff Highlighter visuel dans le DOM.
- [x] **Packet HUD** : Statistiques temps réel in-page.

## 📅 Phase 2 : Industrialisation & Transport (TERMINÉE 🟢)
- [x] **Event Logging** : Archivage SQLite des sessions.
- [x] **PHP Log Bridge** : Redirection des logs serveur vers le client (OpCode `0x10`).
- [x] **Protocol v0.3.1** : Headers universels 4 octets et NodeVersion.
- [x] **Auto-PHP Supervisor** : Détection et lancement automatique de PHP par le Gateway.

## 📅 Phase 3 : DevTools Pro & Diagnostic (En cours 🔵)
*Priorité : Transformer le Dashboard en station de contrôle industrielle.*

### 🏆 Phase 3 : DevTools Pro (v0.3.1) - 80%
1.  **Network Monitor (Binaire)** : **[DONE]** Table live affichant chaque paquet transitant par le Gateway.
2.  **Node Inspector** : **[DONE]** Inspection interactive des noeuds dans le Replay (ID, Version, State).
3.  **State Diff Viewer** : [IN PROGRESS] Visualisation des mutations avant/après.
4.  **HTTP Fallback** : **[DONE]** Transport hybride WebSocket/POST opérationnel.

### 🟡 Accélération Workflow
- [ ] **Handler Tracer** : Timeline verticale (EVENT -> PHP Handler -> PatchOps) pour profiler la performance du backend.
- [ ] **Session Comparator** : Comparer deux sessions côte à côte pour détecter des régressions d'UI.

## 📅 Phase 4 : Déploiement & Optimisation (En attente ⚪)
- [ ] **Payload Tester** : Interface style "Postman" pour injecter des paquets EVENT arbitraires.
- [ ] **Compression Stats** : Rapport détaillé sur les ratios de compression Zstd/Huffman sur les snapshots B-TREE.
- [ ] **VSCode Extension** : Linter et autocomplétion pour `.nhtml`.

---

## 🏗️ État Actuel du Développement
- **Phase 1** : 🟢 **100%**
- **Phase 2** : 🟢 **100%**
- **Phase 3** : 🔵 **20%** (Time Travel & Replay OK)
- **Phase 4** : ⚪ En attente
