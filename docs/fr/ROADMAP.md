# 🗺️ Roadmap NHTML (v0.6.0)

## ✅ Phase 5 — Sécurité & Stabilité (Terminée)
- [x] **Signatures HMAC-SHA256** : Authentification binaire des paquets client.
- [x] **Sequence ID Synchronization** : Anti-replay protocol intégré.
- [x] **WASM Persistence** : Support IDBFS pour Messenger/Settings.
- [x] **Showcase Responsive** : Interface adaptative Desktop/Mobile.

## 🔵 Phase 6 — Performance & Collaboration (Active)
- [x] **FastCGI Client (Rust)** : Bridge direct vers PHP-FPM pool (zéro fork overhead).
- [x] **Scoped Broadcasting** : Synchronisation multi-utilisateur temps réel (`all`, `others`).
- [x] **Error Handling Pro** : Système de `GatewayError` pour une stabilité 100%.
- [ ] **Pool de Connexions** : Réutilisation des sockets FPM (Keep-alive).
- [ ] **Auto-Tunneling** : Intégration Cloudflare Tunnel / Ngrok pour le Cloud Home.
- [ ] **SDKs Communautaires** : Ports du protocole en Python et Go.

---
> **Architecture Cible (Production)** : `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools jamais exposé.

---
> **Architecture Cible (Production)** : `Internet → Nginx → [Gateway:8080 | PHP:8000]` — DevTools jamais exposé.
