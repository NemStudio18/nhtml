# 🛰️ NHTML Protocol Specification (NBPS) v0.7.0
**Édition Performance & Collaboration**

## 1. Structure Universelle des Paquets
Chaque paquet binaire commence par un **Header de 5 octets** :
`[Type: u8] [Length: u32]` (Big-Endian)

---

## 2. Types de Paquets (Header Type)

| Code | Nom | Sens | Description |
|:---|:---|:---|:---|
| `0x01` | **HELLO** | Bidirectionnel | Handshake, Secret & SeqID sync. |
| `0x02` | **EVENT** | Client -> Srv | Interaction signée HMAC-SHA256. |
| `0x03` | **PATCH** | Srv -> Client | Instructions de mutation DOM atomiques. |
| `0x04` | **BIND** | Srv -> Client | Enregistrement de Local Actions. |
| `0x09` | **PING/PONG** | Bidirectionnel | Keep-Alive & Heartbeat. |

---

## 4. Contrat d'Interface Gateway ↔ Backend (v0.6.0)

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

*   **scope** : `all` (tout le monde), `others` (tous sauf l'envoyeur), `group:X` (futur).

---

## 5. Performance & Transport
- **FastCGI Client** : Le Gateway implémente un client FastCGI natif pour parler aux pools PHP-FPM (Port 9000 par défaut).
- **Zstd Compression** : Utilisation du dictionnaire Zstd pour les snapshots B-TREE massifs.
- **Binary Stream** : Toutes les communications sont binaires (sauf le dialogue interne Gateway <-> PHP qui est en JSON streamé sur stdin/stdout).
