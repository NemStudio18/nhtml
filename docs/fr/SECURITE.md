# 🛡️ Sécurité & Limitations Techniques (NHTML)

Ce document détaille les points critiques de sécurité et les limites architecturales du système NHTML. 

---

## 1. Authentification & Intégrité (v0.7.3 ✅)
### Solution implémentée
La sécurité cryptographique obligatoire est activée par défaut :
- **Signatures HMAC-SHA256** : Chaque événement envoyé par le client (`EVENT`) est signé avec une clé secrète de 32 octets négociée lors du handshake. Le Gateway rejette immédiatement toute trame falsifiée.
- **Sequence ID (Anti-Replay)** : Un compteur incrémental est maintenu par session. Le Gateway n'accepte que des paquets avec un `SeqID` supérieur au précédent, rendant les attaques par rejeu impossibles.
- **Marvin Attack** : *Mise à jour (v0.7.3) :* Vulnérabilité (RUSTSEC-2023-0071) résolue suite à la mise à jour de l'arbre des dépendances Rust.

---

## 2. Haute Performance & FastCGI (v0.7.3 ✅)
### Le Risque
En mode CGI classique, le Gateway lance un processus PHP pour chaque événement. Bien que simple, cela peut être exploité pour saturer le CPU (DoS).
### Solution
- **FastCGI (PHP-FPM)** : Le Gateway maintient des connexions persistantes vers un pool de travailleurs PHP. Cela réduit drastiquement l'overhead de création de processus et permet de limiter les ressources au niveau du serveur FPM.
- **Rate Limiting (v0.7.1+)** : Limitation du nombre d'événements par seconde par IP au niveau du Gateway via `[security.rate_limit]`.
- **TLS Natif (v0.7.3+)** : Support du WSS (WebSocket Secure) et HTTPS (`min_version = "1.3"`) directement dans le Gateway sans nécessiter de reverse-proxy.

---

## 3. Limitations du Mode WASM (Zéro-Serveur)
Le mode WASM comporte des limites intrinsèques à la sécurité des navigateurs :
- **Isolation Totale** : Chaque client WASM est une "île". Il n'y a **aucune synchronisation possible** entre deux utilisateurs sur une page statique (GitHub Pages), car il n'y a pas de serveur central pour arbitrer l'état.
- **Bac à Sable (Sandbox)** : PHP-WASM ne peut pas ouvrir de sockets réseau vers l'extérieur. Seul **SQLite local** (via IDBFS) est supporté.
- **Usage** : Ce mode est réservé aux outils "Offline-First" ou aux démonstrations statiques.

---

## 4. Exposition des DevTools
Les DevTools NHTML (`port 8081`) exposent l'intégralité des flux métier :
- **Risque** : Ne **jamais** exposer le port 8081 sur l'Internet public en production.
- **Recommandation** : Utilisez exclusivement un tunnel SSH ou un VPN pour y accéder à distance.

---

## 5. Roadmap Sécurité Future
- **Stockage Session Client** : Migration du session ID de `localStorage` vers `sessionStorage` pour expiration automatique à la fermeture de l'onglet.
- **Monitoring Prometheus** : Intégration complète des métriques d'attaques rejetées vers Grafana.
