# 🛡️ Sécurité & Limitations Techniques (NHTML)

Ce document détaille les points critiques de sécurité et les limites architecturales actuelles du système NHTML. Ces points constituent la priorité de développement pour la version **v0.5.0**.

---

## 1. Gestion des Sessions & Race Conditions
### Le Problème
En mode **Dédié (Rust)**, le Gateway est asynchrone. Si un utilisateur déclenche plusieurs événements simultanément (ex: clics multiples), le Gateway peut lancer plusieurs processus PHP en parallèle.
PHP utilise un verrouillage de fichier (`session_lock`) qui force les requêtes à s'exécuter de manière séquentielle, mais si le code PHP manipule un état persistant (Base de données, Fichiers) sans transactions, des incohérences peuvent apparaître.

### Solution prévue (v0.5.0)
- **Atomic Sequence ID** : Chaque mutation sera associée à un numéro de version d'état. Si le Gateway reçoit une réponse basée sur une version obsolète, elle sera rejetée pour éviter l'écrasement de données.

---

## 2. Intégrité du Protocole (NBPS)
### Risque d'Injection
Le protocole binaire NBPS ne possède actuellement pas de mécanisme d'authentification par paquet. Un attaquant capable de se connecter au WebSocket pourrait injecter ses propres trames binaires pour manipuler le DOM de la victime (XSS Binaire).

### Solution prévue (v0.5.0)
- **Signature HMAC** : Chaque paquet envoyé par le Gateway sera signé avec une clé secrète partagée. Le `bridge.js` vérifiera la signature avant d'appliquer toute mutation.

---

## 3. Limitations du Mode WASM (Zéro-Serveur)
Le mode WASM est révolutionnaire mais comporte des limites intrinsèques à la sécurité des navigateurs :
- **Isolation Totale** : Chaque client WASM est une "île". Il n'y a **aucune synchronisation possible** entre deux utilisateurs sur une page statique (GitHub Pages), car il n'y a pas de serveur central pour arbitrer l'état.
- **Bac à Sable (Sandbox)** : PHP-WASM ne peut pas ouvrir de sockets réseau (TCP/UDP) vers l'extérieur. Les connexions aux bases de données distantes (MySQL, PostgreSQL) sont impossibles. Seul **SQLite local** (en RAM ou IndexedDB) est supporté.
- **Usage** : Ce mode est réservé aux outils "Offline-First", aux calculatrices ou aux démonstrations.

---

## 4. Exposition des DevTools
Les DevTools NHTML (`port 8081`) sont un outil de diagnostic puissant mais dangereux :
- **Visibilité totale** : Ils exposent l'intégralité des flux métier et des structures de données.
- **Risque** : Ne **jamais** exposer le port 8081 sur l'Internet public en production.
- **Recommandation** : Utilisez exclusivement un tunnel SSH pour y accéder à distance.
