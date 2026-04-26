# 🗺️ Roadmap NHTML (v0.4.0)

## ✅ Phase 1 — Fondations & Glow (Terminée)
- [x] NHTML Glow, Packet HUD

## ✅ Phase 2 — Transport Industriel (Terminée)
- [x] SQLite Sessions, PHP Log Bridge, Protocol v0.3.1, Auto-PHP Supervisor

## ✅ Phase 3 — DevTools Pro (Terminée)
- [x] Network Monitor, Node Inspector, State Diff Viewer, Handler Tracer, Session Comparator

## 🔵 Phase 4 — Optimisation & Exemples (Active)
- [x] **Compression Zstd** : B-TREE compressé + décompression client (fzstd local, zéro CDN)
- [x] **Benchmark CLI** : `nhtml bench <fichier>`
- [x] **Auto-Injection Bridge** : `bridge.js` + `fzstd.js` injectés automatiquement dans les `.nhtml`
- [x] **Ports Configurables** : `--ws-port`, `--php-port`, `--port` (devtools)
- [x] **Pyramide d'Exemples** : counter → todo → live-form → style-lab
- [ ] **Packaging v0.4.0** : Binaires Win/Linux/Mac + PHP portable inclus dans le bundle

## 🔜 Phase 5 — Release Publique
- [x] **Monorepo Unifié** : Un seul dépôt `nhtml-gateway` regroupant Rust, PHP SDK et JS Bridge.
- [x] PHP-WASM : Fallback "Zéro-Serveur" intégré dans le Bridge JS (Local & Sans CDN)
- [x] `nhtml.config.toml` : Configuration déclarative des ports et chemins
- [x] Reverse Proxy Doc : Config Nginx/Apache prête à copier-coller
- [ ] GitHub Pages : Landing page + docs publiques

---
> **Architecture Cible (Production)** : `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools jamais exposé.
