# 🛰️ NHTML Protocol Specification (NBPS) v0.5.0
**Édition Sécurité Industrielle**

## 1. Structure Universelle des Paquets
Chaque paquet binaire commence par un **Header de 5 octets** :
`[Type: u8] [Length: u32]` (Big-Endian)

Le champ `Length` représente la taille totale du payload (excluant le header).

---

| `0x01` | **HELLO** | Bidirectionnel | Handshake, Sync Session & Version. Inclut désormais un `Challenge` de 32 octets. |
| `0x02` | **EVENT** | Client -> Srv | Interaction utilisateur. Inclut désormais `SequenceID` (4 octets) + `HMAC` (32 octets). |
| `0x03` | **PATCH** | Srv -> Client | Instructions de mutation DOM atomiques. |
| `0x04` | **BIND** | Srv -> Client | Enregistrement de Local Actions & Handlers. |
| `0x05` | **SYNC** | Srv -> Client | Vérification d'intégrité (Checksum comparison). |
| `0x07` | **B-TREE** | Srv -> Client | Snapshot DOM complet avec NIDs (Mapping inclus). |
| `0x09` | **PING/PONG** | Bidirectionnel | Keep-Alive & Heartbeat (0xFF = Force Reload). |
| `0x10` | **LOG** | Srv -> Client | Relai des logs console backend. |
| `0x7F` | **ERR** | Srv -> Client | Rapport d'erreur structuré binaire. |

---

## 2.5 Détail de la Sécurité (v0.5.0)

### Format du Paquet EVENT (0x02)
`[Type: 1b] [Length: 4b] [SeqID: 4b] [Signature: 32b] [HandlerLen: 1b] [Handler: ...] [PayloadLen: 2b] [Payload: ...]`

*   **SeqID** : Entier incrémentiel. Le serveur rejette tout paquet `<= lastSeqID`.
*   **Signature** : HMAC-SHA256 calculé sur `[Type + Length + SeqID + Handler + Payload]` en utilisant le `SessionSecret`.

### Format du Handshake HELLO (0x01)
`[Type: 1b] [Length: 4b] [SessionID: str16] [Version: u16] [SessionSecret: 32b]`
*Le `SessionSecret` est généré par le Gateway et utilisé par le Bridge pour signer tous les événements futurs.*

---

## 3. DOM Operations (OpTypes in 0x03 PATCH)

Structure d'une mutation dans un paquet PATCH :
`[TargetID: u16] [OpType: u8] [Version: u32] [DataLen: u16] [Data: bytes]`

| Code | Opération | Paramètre(s) |
|:---|:---|:---|
| `0x01` | **SET_TEXT** | string16 |
| `0x02` | **SET_ATTR** | string8 (key) + string16 (val) |
| `0x03` | **DEL_ATTR** | string8 (key) |
| `0x04` | **ADD_CLASS** | string16 (class) |
| `0x05` | **DEL_CLASS** | string16 (class) |
| `0x06` | **INSERT_BEFORE**| u16 (refId) + string16 (HTML) |
| `0x07` | **INSERT_AFTER** | u16 (refId) + string16 (HTML) |
| `0x08` | **REMOVE** | (none) |
| `0x09` | **SET_STYLE** | string8 (prop) + string16 (val) |
| `0x0A` | **REPLACE_INNER**| string16 (HTML) |
| `0x0B` | **APPEND_HTML** | string16 (HTML) |
| `0x0C` | **SCROLL_TO** | (none) |
| `0x0D` | **FOCUS** | (none) |

---

## 4. Contrat d'Interface Gateway ↔ Backend (JSON Generic)

Le Gateway Rust est totalement générique. Il mappe les `nid` (strings) en `node_id` (u16) dynamiquement.

### Requete (Gateway -> PHP/CGI)
Le Gateway transmet l'événement via les variables d'environnement (CGI) ou via un payload JSON structuré :
```json
{
  "nhtml_event": "click",
  "node_id": 12,
  "session_id": "uuid-...",
  "payload": { ... }
}
```

### Réponse (PHP -> Gateway)
Le SDK PHP utilise des noms d'opérations en `snake_case` pour générer les patchs.
```json
{
  "patch": [
    { "op": "set_text", "nid": "counter", "val": "42" },
    { "op": "add_class", "nid": "header", "val": "active" }
  ]
}
```

---

## 5. Mécanisme SYNC (Checksum)
Périodiquement, le serveur envoie `0x05 [Checksum: u32]`.
1. Le client calcule son `local_checksum` (ID + Value length).
2. Si `local_checksum != server_checksum`, le client émet un `HELLO` (0x01).
3. Le serveur répond par un `B-TREE` (0x07) pour restaurer l'état.

---

## 6. Format B-TREE (Mapping v0.4.0)
Chaque nœud dans le B-TREE inclut désormais son `tag` (NID string).
`[Count: u16] { [ID: u16] [Version: u32] [TagLen: u8] [Tag: str8] [ValLen: u16] [Val: str16] }`

---

## 7. Performance & Compression
- **Zstd** est utilisé par défaut sur les paquets `B-TREE`.
- **IIFE Guard** dans `bridge.js` pour éviter les doubles initialisations.
- **Node Caching** côté client pour des mutations ultra-rapides (< 1ms).
