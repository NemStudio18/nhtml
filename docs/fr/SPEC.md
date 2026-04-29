# 🛰️ NHTML Protocol Specification (NBPS) v0.7.3
**Édition Sécurité & Stabilité**

## 1. Structure Universelle des Paquets
Chaque paquet binaire commence par un **Header de 5 octets** :
`[Type: u8] [Length: u32]` (Big-Endian)

---

## 2. Types de Paquets (Header Type)

| Code | Nom | Sens | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bidirectionnel | Handshake, synchronisation du Secret HMAC et SeqID. |
| `0x02` | **EVENT** | Client -> Srv | Interaction utilisateur (ex: clic, saisie) signée via HMAC-SHA256. |
| `0x03` | **PATCH** | Srv -> Client | Instructions de mutation DOM atomiques (ajout, modif, suppression). |
| `0x07` | **B-TREE** | Srv -> Client | Snapshot complet du DOM compressé via Zstd. |
| `0x08` | **PUSH_PATCH**| Client -> Srv | Relai de patch local (Mode Zero-Server / Collaboration en temps réel). |
| `0x09` | **PING/PONG** | Bidirectionnel | Keep-Alive et maintien de la connexion (Heartbeat). |
| `0x10` | **LOG** | Srv -> Client | Remontée de logs système vers les DevTools du client. |

---

## 3. Détails des Payloads

### 0x01 - HELLO
`[Type:1][Len:4][Status:1][SidLen:1][SessionId...][Secret:32][LastSeq:4]`

### 0x02 - EVENT
`[Type:1][Len:4][SeqId:4][Signature:32][NodeID:4][HLen:1][Handler:str][PLen:2][Payload:json]`

### 0x03 - PATCH
`[Type:1][Len:4][OpCount:2]` suivi d'une liste d'opérations :
`[TargetID:2][OpTypeCode:1][Version:4][DataLen:2][Value...]`

### 0x07 - B-TREE
`[Type:1][Len:4][Compression:1][OrigLen:4][Checksum:4][NodeCount:2][ZstdData...]`

### 0x08 - PUSH_PATCH
Identique au PATCH (`0x03`) mais émis par le client, limité à 64 opérations maximum par message pour prévenir la surcharge du thread de rendu.

### 0x10 - LOG
`[Type:1][Len:4][Severity:1][MsgLen:2][Message...]`

---

## 4. Contrat d'Interface Gateway ↔ Backend (PHP)

Le Gateway communique avec le PHP via **CGI (Standard)** ou **FastCGI (Performance)**.

### Réponse PHP (Format JSON)
Le SDK PHP peut retourner un tableau simple de patchs, ou un objet structuré pour inclure des instructions de diffusion.

```json
{
  "patch": [
    { "op": "set_text", "nid": "status", "val": "Connecté" }
  ],
  "broadcast": {
    "scope": "others",
    "patch": [
      { "op": "append_html", "nid": "logs", "val": "<li>User-42 a cliqué !</li>" }
    ]
  }
}
```

*   **scope** : `all` (tout le monde), `others` (tous sauf l'envoyeur), `room:X` (groupe localisé), `direct:X` (message privé).

---

## 5. Performance, Sécurité & Transport
- **Sécurité CSWH** : Validation stricte des origines via liste blanche (`allowed_origins` dans `nhtml.config.toml`).
- **Limitation de Taux (Rate Limiting)** : Cache O(1) de type LRU par adresse IP (Limitation par défaut à 30 events/sec pour contrer le DoS).
- **Zstd Compression** : Utilisation du dictionnaire Zstd pour les snapshots B-TREE massifs.
- **Binary Stream** : Toutes les communications sont binaires, encodées en stricte vérification UTF-8 pour les chaînes de caractères.
