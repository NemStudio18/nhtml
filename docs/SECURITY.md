# NHTML Security Information

## 🛡️ Production Hardening (v0.7.3-stable)

The v0.7.3 release includes a major security overhaul to ensure NHTML is ready for production environments.

### 1. XSS Protection (CRIT-01)
- **DOMPurify Integration**: The `bridge.js` now strictly sanitizes all HTML content before injecting it into the DOM.
- **Opcode Level Sanitization**: This affects `REPLACE_INNER`, `APPEND_HTML`, `INSERT_BEFORE`, and `INSERT_AFTER`.
- **Fallback Policy**: If DOMPurify is missing, an aggressive manual sanitizer is used (stripping scripts, iframes, objects, and dangerous attributes like `on*` or `javascript:`).

### 2. CSWH Protection (HIGH-03)
- **Cross-Origin WebSocket Hijacking**: The Gateway now performs strict Origin verification.
- **Allowed Origins**: You must configure `allowed_origins` in `nhtml.config.toml` to restrict which domains can connect to the WebSocket gateway.
- **Default Policy**: In development mode, `localhost` and `127.0.0.1` are allowed. In production, an empty or missing Origin header will result in a 403 Forbidden error if `allowed_origins` is set.

### 3. Path Traversal (HIGH-01)
- **Resolved Path Enforcement**: Both the Rust Gateway and the PHP Router use `canonicalize()` and `realpath()` to resolve paths.
- **Root Jail**: All file inclusions and static file serves are strictly checked to ensure they stay within the configured project root.

### 4. Event Integrity (NBPS Protocol)
- **HMAC-SHA256 Signatures**: Every EVENT packet from the client is signed with a session-specific secret.
- **Sequence IDs**: Prevents replay attacks by ensuring each event has a strictly increasing sequence number.

### 5. Config Security (CONFIG-01)
- **Resolution Order**: `$NHTML_CONFIG` (env) > `./nhtml.config.toml` > `{exe_dir}/nhtml.config.toml`.
- **Validation**: The binary will immediately exit with a fatal error if the configuration file is malformed, preventing accidental insecure defaults.

---

## 🛡️ Durcissement Production (v0.7.3-stable) [FR]

La version v0.7.3 inclut une refonte majeure de la sécurité pour garantir que NHTML est prêt pour la production.

### 1. Protection XSS (CRIT-01)
- **Intégration DOMPurify** : Le `bridge.js` sanitise désormais strictement tout le contenu HTML avant injection.
- **Sanitisation au niveau Opcode** : S'applique à `REPLACE_INNER`, `APPEND_HTML`, `INSERT_BEFORE`, et `INSERT_AFTER`.
- **Fallback Agressif** : Si DOMPurify est absent, un sanitiseur manuel dépouille le contenu de toute balise dangereuse (scripts, iframes, etc.) et attributs `on*`.

### 2. Protection CSWH (HIGH-03)
- **Cross-Origin WebSocket Hijacking** : La Gateway effectue une vérification stricte de l'en-tête `Origin`.
- **Whitelist d'Origines** : Configurez `allowed_origins` dans `nhtml.config.toml` pour restreindre les domaines autorisés.
- **Politique par défaut** : En mode dev, `localhost` est autorisé. En production, un Origin manquant ou non listé entraîne une erreur 403.

### 3. Path Traversal (HIGH-01)
- **Résolution Stricte** : La Gateway et le routeur PHP utilisent `canonicalize()` et `realpath()`.
- **Projet "Jail"** : Toute inclusion est vérifiée pour s'assurer qu'elle reste dans le répertoire racine du projet.

### 4. Intégrité des Événements (Protocole NBPS)
- **Signatures HMAC-SHA256** : Chaque paquet EVENT est signé avec un secret unique par session.
- **Sequence IDs** : Empêche les attaques par rejeu via une numérotation stricte des événements.

### 5. Sécurité de Configuration (CONFIG-01)
- **Ordre de Résolution** : `$NHTML_CONFIG` (env) > `./nhtml.config.toml` > `{exe_dir}/nhtml.config.toml`.
- **Validation Fatale** : Le binaire s'arrête immédiatement si le fichier de config est invalide, évitant tout mode dégradé non sécurisé.

---

## Known Vulnerabilities (Resolved)

### RSA Marvin Attack (RUSTSEC-2023-0071)
*Update (v0.7.3):* This vulnerability has been officially resolved in the latest dependency tree update. NHTML Gateway v0.7.3 and newer are no longer affected.

---

© 2026 NemStudio18 — Security First.
