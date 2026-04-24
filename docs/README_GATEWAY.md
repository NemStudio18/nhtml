# 🛰️ NHTML Gateway

**Le relai binaire haute-performance pour les applications NHTML.**

Le Gateway NHTML est un serveur WebSocket écrit en Rust, conçu pour faire le pont entre les clients web (Binary NBPS) et les backends PHP traditionnels. Il transforme les mutations DOM binaire en requêtes d'état ultra-rapides.

## 🚀 Fonctionnalités Clés
- **Binary Relay (NBPS)** : Gestion native du protocole binaire pour une latence minimale.
- **PHP Supervisor** : Lance et surveille automatiquement vos processus PHP en mode local.
- **MPSC Architecture** : Gestion robuste des connexions multiples via des canaux asynchrones (Tokio).
- **Zstd Compression** : Compression à la volée des snapshots B-TREE pour économiser jusqu'à 70% de bande passante.
- **SQLite Persistence** : Archivage automatique des sessions pour le replay et le diagnostic.

## 📦 Installation (Binaires)
Aucune compilation n'est requise pour l'utilisation standard.
1. Téléchargez le binaire correspondant à votre OS depuis les [Releases](https://github.com/NemStudio18/nhtml-gateway/releases).
2. Placez le binaire dans le dossier de votre projet PHP.
3. Lancez : `./nhtml start --dev` (votre app sera dispo sur `http://127.0.0.1:3000`)

## 💻 Commandes CLI

Le binaire `nhtml` expose plusieurs commandes pour s'adapter à votre workflow :

- **`nhtml start`** : Lance le Gateway WebSocket, le serveur HTTP (port 3000, avec auto-injection du bridge NHTML), et le superviseur PHP.
  - `--dev` : Active l'auto-rechargement (live-reload).
  - `--ws-port <port>` : Force le port du WebSocket (défaut: 8080).
  - `--php-port <port>` : Force le port du backend PHP (défaut: 8000).
- **`nhtml devtools`** : Lance le tableau de bord de diagnostic.
  - `--port <port>` : Définit le port du serveur DevTools (défaut: 8081).
- **`nhtml bench <fichier.nhtml>`** : Compare les performances de transmission (Taille HTML brute vs Taille du patch NBPS).

## 🛠️ Développement
Si vous souhaitez compiler le gateway vous-même :
```bash
cargo build --release
```

## 📜 Licence
Ce composant est sous licence **AGPL v3**. 
Pour des besoins commerciaux ou des déploiements cloud propriétaires, contactez NemStudio.
