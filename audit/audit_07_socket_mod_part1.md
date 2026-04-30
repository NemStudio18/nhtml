---
# 🔍 Audit — `src/socket/mod.rs` (Partie 1: Réseau & Sécurité)
**Fichier d'audit** : `audit_07_socket_mod_part1.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_08_socket_mod_part2.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/socket/mod.rs` (Lignes 1 - 800) |
| Rôle | Cœur réseau : WebSockets, HTTP, Rate Limiting, Protection CSWH |
| Lignes | ~1631 au total |
| Langage | Rust |
| Score Sécurité | 🟠 5/10 |
| Score Performance | 🟢 9/10 |

---

## 🟠 VULNÉRABILITÉS HAUTES
> Corriger en URGENCE absolue

### HIGH-07-001 — Protection CSWH contournable (Cross-Site WebSocket Hijacking)
- **Ligne(s)** : L.513
- **Type** : Contournement d'autorisation (OWASP A01:2021)
- **Impact** : La validation de l'Origine utilise la méthode `.contains(host)` pour vérifier si la requête provient bien du domaine légitime. C'est une faille classique !
  Si le domaine légitime (`host`) est `monsite.com`, un attaquant peut enregistrer le domaine malveillant `http://attaquant-monsite.com` ou `http://monsite.com.attaquant.net`. La condition `.contains("monsite.com")` renverra `true`, permettant à l'attaquant de détourner la session WebSocket de l'utilisateur.

**Code vulnérable** :
```rust
if !origin_str.is_empty() && !origin_str.contains(host) && !origin_str.contains("localhost") {
    // Rejet
}
```

**Code corrigé** :
```rust
// Doit correspondre EXACTEMENT au host ou appartenir à la liste stricte `allowed_origins`.
// Parser les URL pour comparer précisément le nom de domaine.
```

---

## 🟡 VULNÉRABILITÉS MOYENNES
### MED-07-001 — Fonction de hachage vulnérable pour les logs
- **Ligne(s)** : L.37-42
- **Type** : Cryptographie faible
- **Impact** : `DefaultHasher` dans Rust (SipHash) est rapide mais n'est pas conçu pour résister aux attaques cryptographiques s'il est prédictible. Même s'il n'est utilisé que pour logger (`hash_sid`), une collision intentionnelle massive (HashDoS) pourrait théoriquement noyer les logs ou compliquer le traçage d'une attaque. Il vaut mieux utiliser une empreinte SHA256 tronquée.

---

## ✅ POINTS POSITIFS
- **Rate Limiter Mem-Bounded** : L'implémentation du Rate Limiter (L.429) via `lru::LruCache` est brillante ! Elle plafonne la mémoire à 2048 IPs, empêchant totalement un attaquant de saturer la RAM (OOM) en générant des millions de fausses adresses IP (IP spoofing).
- **Anti-Path Traversal** : L'utilisation de `std::fs::canonicalize` couplé à `.starts_with(&root_canonical)` (L.536-547) pour servir les fichiers statiques HTTP est blindée contre les attaques par retour de répertoire (`../../etc/passwd`).

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 1 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 0 |
| ⚪ Qualité | 0 |

---
*Fichier suivant → `audit_08_socket_mod_part2.md`*
