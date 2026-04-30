---
# 🔍 Audit — `nhtml.config.toml`
**Fichier d'audit** : `audit_02_nhtml_config.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_03_release_yml.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `nhtml.config.toml` |
| Rôle | Configuration globale par défaut du Gateway |
| Lignes | 23 |
| Langage | TOML |
| Score Sécurité | 🟡 7/10 |
| Score Performance | 🟢 10/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-02-001 — Omission des directives de sécurité par défaut
- **Ligne(s)** : L.1–23
- **Type** : Security Misconfiguration (OWASP A05:2021)
- **Impact** : Le fichier de configuration par défaut ne liste pas les options de sécurité telles que `allowed_origins` (CORS/CSWH) ou les limites du rate limiter (`rate_limit`). Les utilisateurs finaux pourraient ignorer l'existence de ces protections essentielles et déployer l'outil avec des configurations permissives.

**Avant** :
```toml
# Seulement ports, dev, et fastcgi présents
[ports]
ws = 8080
```

**Après** :
```toml
[ports]
ws = 8080

[security]
# Limite le nombre de paquets par IP par seconde (Protection DoS)
rate_limit = 30
# Liste blanche des domaines autorisés pour prévenir le CSWH (Cross-Site WebSocket Hijacking)
allowed_origins = ["http://localhost:3000", "https://mon-domaine.com"]
```

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [x] Clarté : Les commentaires sont clairs et explicites pour chaque bloc existant.
- [ ] Omission de l'URL Redis : La section `[cluster]` est absente du template par défaut, ce qui peut rendre complexe la découverte de la fonctionnalité de scalabilité horizontale pour les nouveaux développeurs.

---

## ✅ POINTS POSITIFS
- Pas de secrets hardcodés (mots de passe, tokens) présents dans le fichier.
- Le port des DevTools (8082) est paramétrable, ce qui permet d'éviter les conflits de ports locaux.

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 0 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 0 |
| ⚪ Qualité | 1 |

---
*Fichier suivant → `audit_03_release_yml.md`*
