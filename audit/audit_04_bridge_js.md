---
# 🔍 Audit — `assets/js/bridge.js`
**Fichier d'audit** : `audit_04_bridge_js.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_05_main_rs.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `assets/js/bridge.js` |
| Rôle | Client frontend, parseur binaire, DOM Patcher |
| Lignes | ~1037 |
| Langage | JavaScript (ES6) |
| Score Sécurité | 🟠 6/10 |
| Score Performance | 🟡 8/10 |

---

## 🟠 VULNÉRABILITÉS HAUTES
> Corriger sous 2 semaines

### HIGH-04-001 — Risque de DOM-based XSS via `innerHTML` et `insertAdjacentHTML`
- **Ligne(s)** : L.462, L.466, L.775, L.787
- **Type** : Cross-Site Scripting (XSS) (OWASP A03:2021)
- **Impact** : Le parseur binaire et JSON traite les opcodes `REPLACE_INNER` (0x0A) et `APPEND_HTML` (0x0B) en utilisant l'assignation directe `el.innerHTML = ...` ou `insertAdjacentHTML`. Si le serveur (ou un module PHP malveillant/défaillant) envoie un patch contenant `<script>malicious()</script>` ou `<img src=x onerror=alert(1)>`, cela sera exécuté dans le navigateur de l'utilisateur final. Bien que le Gateway soit la source de confiance, confier la sanitisation exclusivement au backend est une faille de "défense en profondeur".
- **Exploitabilité** : Moyenne (Nécessite la compromission du backend PHP ou l'injection de payload dans un broadcast non sanitisé).

**Code vulnérable** :
```javascript
case 0x0A: // REPLACE_INNER
    el.innerHTML = new TextDecoder().decode(data.slice(2));
    break;
```

**Code corrigé** :
```javascript
case 0x0A: // REPLACE_INNER
    // Implémenter DOMPurify ou Trusted Types pour sécuriser l'injection
    const rawHTML = new TextDecoder().decode(data.slice(2));
    el.innerHTML = window.DOMPurify ? DOMPurify.sanitize(rawHTML) : rawHTML;
    break;
```
*Note : Le même problème s'applique à la méthode `applyJsonPatch()` pour le mode WASM local.*

---

## 🟡 VULNÉRABILITÉS MOYENNES
### MED-04-001 — Crypto : Non-rotation de l'IV (Nonce) pour HMAC (si chiffré)
- **Ligne(s)** : L.352-359
- **Type** : Pratique cryptographique
- **Impact** : L'implémentation HMAC (WebCrypto API) pour la vérification d'intégrité est excellente. Cependant, si dans le futur les payloads sont chiffrés avec cette clé, l'absence de vecteur d'initialisation (IV) dynamique par paquet poserait un problème critique. À surveiller.

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-04-001 — Layout Thrashing dans `nhtml_run_mousemove_actions`
- **Ligne(s)** : L.229
- **Type** : Reflow forcé (Layout Thrashing)
- **Impact** : L'appel à `entry.el.getBoundingClientRect()` à l'intérieur d'un événement `mousemove` (déclenché 60 fois par seconde) provoque un recalcul synchrone du layout, pouvant causer des drops de frames sur des interfaces complexes.

**Avant** :
```javascript
const rect = entry.el.getBoundingClientRect(); // Lent dans un mousemove
x -= rect.left; y -= rect.top;
```

**Après** :
```javascript
// Mettre en cache la BoundingBox via un IntersectionObserver ou lors du resize
const rect = window.nhtml_rect_cache.get(entry.el) || entry.el.getBoundingClientRect();
x -= rect.left; y -= rect.top;
```

---

## ✅ POINTS POSITIFS
- **Zstd Natif** : L'utilisation de `fzstd` pour décompresser les snapshots d'arbres DOM binaires est un gain majeur en bande passante.
- **HMAC Intégré** : L'utilisation de `crypto.subtle` pour HMAC-SHA256 prévient les falsifications d'événements (Tampering).
- **Architecture Zero-Server** : L'encapsulation élégante de WASM-PHP en fallback.

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 1 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 1 |
| ⚪ Qualité | 0 |

---
*Fichier suivant → `audit_05_main_rs.md`*
