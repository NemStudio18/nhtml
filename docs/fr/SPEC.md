# 🛰️ NHTML Protocol Specification (NBPS) v0.7.4
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

## 5. Performance, Sécurité & Résilience (v0.7.4)
- **Delta Sync (Nouveauté v0.7.4)** : Lors d'une reconnexion, le Gateway compare le `LastSeq` du client. Si les patchs manquants sont en cache (`delta_history`), ils sont rejoués au lieu de renvoyer un `B-TREE` complet.
- **Circuit Breaker (Nouveauté v0.7.4)** : Protection contre la saturation. Si un backend PHP-FPM échoue 10 fois de suite, le Gateway "ouvre le circuit" et rejette les requêtes pendant 10 secondes pour permettre au backend de récupérer.
- **Adaptive Zstd Compression** : Le niveau de compression varie dynamiquement de 3 (rapide) à 12 (fort) selon la taille du payload. Les paquets < 256 octets ne sont pas compressés.
- **SQLite WAL Mode** : Utilisation systématique du mode *Write-Ahead Logging* pour permettre des écritures concurrentes sans bloquer les lectures de sessions.
- **Sécurité CSWH** : Validation stricte des origines via liste blanche (`allowed_origins` dans `nhtml.config.toml`).
- **Limitation de Taux (Rate Limiting)** : Cache O(1) de type LRU par adresse IP (Limitation par défaut à 30 events/sec).
- **Binary Stream** : Toutes les communications sont binaires, encodées en stricte vérification UTF-8.

