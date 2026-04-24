# 🛰️ NHTML Protocol Specification (NBPS) v0.3.1
**Source de Vérité Unique**

## 1. Structure Universelle des Paquets
Chaque paquet binaire commence par un **Header de 5 octets** :
`[Type: u8] [Length: u32]` (Big-Endian)

Le champ `Length` représente la taille totale du payload (excluant le header).

---

## 2. Table des OpCodes (Définitivie v0.3.1)

| Code | Nom | Direction | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bidirectionnel | Handshake, Sync Session & Version. |
| `0x02` | **EVENT** | Client -> Srv | Interaction utilisateur (Clic, Input). |
| `0x03` | **PATCH** | Srv -> Client | Instructions de mutation DOM atomiques. |
| `0x04` | **BIND** | Srv -> Client | Enregistrement de Local Actions & Handlers. |
| `0x06` | **PING/PONG** | Bidirectionnel | Mesure de latence (Round Trip Time). |
| `0x07` | **B-TREE** | Srv -> Client | Snapshot DOM complet (Snapshot/Recovery). |
| `0x09` | **RELOAD** | Srv -> Client | Signal de Hot Reload pour DevTools. |
| `0x0B` | **APPEND_HTML**| Srv -> Client | Ajout incrémental de contenu (Logs/Monitor). |
| `0x10` | **LOG** | Srv -> Client | Relai des logs console backend. |

---

## 3. Contrat d'Interface Gateway ↔ Backend

### Mode Dédié (Standard)
Le Gateway Rust agit comme un traducteur universel.
- **Input (Navigateur -> Gateway)** : Binaire NBPS (0x02 EVENT).
- **Relai (Gateway -> Backend)** : **JSON POST**.
    - Payload : `{"nhtml_event": "click", "node_id": "btn_increment", "current_state": {...}}`
- **Output (Backend -> Gateway)** : **JSON Array** (Liste de patches).
- **Envoi (Gateway -> Navigateur)** : Binaire NBPS (0x03 PATCH).

### Mode Mutualisé (Fallback HTTP)
Pour les serveurs PHP sans Gateway Rust.
- **Contrat** : Binaire NBPS pur. Le Backend (ex: `router.php`) doit décoder le binaire manuellement et renvoyer une frame `0x03` binaire.

---

## 4. Gestion des Conflits (NodeVersion)
Chaque mutation (`0x03`) ou snapshot (`0x07`) porte un `NodeVersion (u32)`.
1.  **Incrémentation** : Seul le serveur incrémente la version d'un nœud.
2.  **Validation Client** : 
    - Si `Version_Recue > Version_Locale` : Appliquer le patch.
    - Si `Version_Recue <= Version_Locale` : Ignorer (Paquet obsolète ou déjà traité).
    - Si saut de version majeur : Demander un `B-TREE` (0x07).

---

## 5. Limitations par Mode de Transport
- **WebSocket (WS)** : Full-Duplex. Supporte le `Push` serveur (LOG, RELOAD, N-LIVE).
- **HTTP POST (Fallback)** : Request/Response uniquement. Pas de logs en temps réel, pas de Hot Reload.

---

## 6. Format des Paquets Clés

### 0x03 PATCH
`[0x03] [OpCount: u16] [ (TargetID: u16, OpType: u8, Version: u32, Data: string16) ... ]`

### 0x07 B-TREE (Snapshot)
`[0x07] [TotalLen: u32] [Compression: u8] [OrigLen: u32] [Checksum: u32] [Payload]`
- **Compression (u8)** : `0x00` (None), `0x01` (Zstd - Défaut v0.4.0).
- **OrigLen (u32)** : Taille du payload avant compression.
- **Checksum (u32)** : CRC32 du payload **original**.
- **Payload** : Données sérialisées (compressées si `0x01`).

---

## 7. Benchmark & Performance
Le protocole NBPS v0.4.0 intègre un outil de mesure de performance intégré via `nhtml bench`.
- **Gain moyen attendu** : 40% à 70% par rapport au HTML brut.
- **Overhead Header** : 5 octets par paquet.
