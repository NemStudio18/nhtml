# 📑 NHTML Protocol Internals & Roadmap

Ce document explique le rôle des fonctions et constantes générant des avertissements de compilation (warnings) et détaille la structure du protocole NBPS v0.3.1.

## 🛠️ Fonctions & Constantes "Warnings"

Le compilateur Rust signale plusieurs éléments comme "inutilisés". Ils sont conservés car ils définissent le squelette des prochaines phases de développement.

### 1. Gestion des Local Actions (LA_*)
*   **Constantes** : `LA_TRIG_HOVER`, `LA_TRIG_SCROLL`, `LA_TRIG_DRAG`, `LA_FLAG_REVERSE_LEAVE`, etc.
*   **But** : Permettre au serveur d'envoyer des instructions au client pour gérer des animations ou des interactions sans repasser par le réseau (ex: "ajoute cette classe CSS quand on survole ce bouton").
*   **Statut** : Le moteur Rust sait générer les paquets, mais le `bridge.js` n'a pas encore le runner d'actions complet.

### 2. Handshake & Lifecycle
*   **`proto::hello`** : Destiné à envoyer un acquittement binaire après la connexion. Actuellement, le WebSocket `onopen` suffit. Sera utilisé pour négocier la compression Zstd.
*   **`proto::ping`** : Destiné à mesurer la latence applicative précise (Round Trip Time applicatif). Actuellement, on utilise les timestamps du monitoring.
*   **`proto::err`** : Destiné à envoyer des codes d'erreurs normalisés au client.

### 3. Binding Dynamique (`proto::bind`)
*   **Structures** : `BindParams`, `LocalActionEntry`.
*   **But** : Permettre l'enregistrement de nouveaux nœuds injectés dynamiquement dans le DOM par JavaScript, tout en les rattachant au cycle de vie NHTML.

---

## 📡 Structure des Paquets (v0.3.1)

Chaque paquet commence par un **Header de 5 octets** : `[Type:1][Len:4]`.

| Type | Nom | Direction | Usage |
| :--- | :--- | :--- | :--- |
| `0x01` | HELLO | Bidirectionnel | Handshake et négo version |
| `0x02` | EVENT | Client -> Srv | Clic, Input, etc. |
| `0x03` | PATCH | Srv -> Client | Mutations DOM |
| `0x07` | B-TREE | Srv -> Client | Snapshot DOM complet |
| `0x09` | RELOAD | Srv -> Client | Rafraîchissement DevTools |
| `0x0B` | APPEND_HTML | Srv -> Client | Ajout incrémental (Logs/Moniteurs) |
| `0x10` | LOG | Srv -> Client | Mirroring Console F12 |

### Nouveau : OpCode `0x0B` (APPEND_HTML)
Crucial pour le **Network Monitor**. Contrairement au `ReplaceInner` (0x0A), il utilise `insertAdjacentHTML('beforeend', ...)` pour accumuler les données sans détruire les nœuds existants.
