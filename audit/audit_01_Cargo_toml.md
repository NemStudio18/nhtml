---
# 🔍 Audit — `Cargo.toml`
**Fichier d'audit** : `audit_01_Cargo_toml.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_02_nhtml_config.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `Cargo.toml` |
| Rôle | Gestionnaire de paquets et configuration du projet Rust |
| Lignes | 48 |
| Langage | TOML |
| Score Sécurité | 🟡 8/10 |
| Score Performance | 🟢 9/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-01-001 — Vulnérabilité transitive "Marvin Attack" via `sqlx-mysql`
- **Ligne(s)** : L.26
- **Type** : Canal auxiliaire temporel (Timing side-channel)
- **Référence** : RUSTSEC-2023-0071
- **Impact** : L'inclusion de la feature `mysql` dans `sqlx` 0.8 importe la crate `rsa` (0.9.x) qui est vulnérable à l'attaque Marvin. Un attaquant MITM pourrait théoriquement récupérer des clés privées via des milliers de mesures temporelles lors du handshake d'authentification MySQL.
- **Exploitabilité** : Complexe (nécessite un accès MITM et un volume de requêtes très élevé).

**Code vulnérable** :
```toml
# L'inclusion de "mysql" tire la dépendance rsa vulnérable
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "sqlite", "mysql", "postgres", "macros", "chrono", "uuid"] }
```

**Code corrigé** :
```toml
# Si MySQL n'est pas strictement requis en production, retirer les drivers non utilisés.
# Sinon, isoler la base de données via un VPC et/ou forcer le TLS (déjà documenté dans SECURITY.md).
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "sqlite", "macros", "chrono", "uuid"] }
```

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-01-001 — Fonctionnalités inutilisées dans `tokio`
- **Ligne(s)** : L.15
- **Type** : Bloat binaire / Compilation plus lente
- **Impact** : Temps de compilation accru et binaire légèrement plus lourd.

**Avant** :
```toml
tokio = { version = "1.0", features = ["full"] }
```

**Après** :
```toml
# Activer uniquement les features réellement utilisées (ex: "rt-multi-thread", "net", "time", "macros", "fs")
tokio = { version = "1.0", features = ["rt-multi-thread", "net", "time", "macros", "sync", "fs"] }
```

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [x] Versions explicites : La plupart des dépendances utilisent des versions mineures strictes (ex: `0.16`), ce qui est bien.
- [ ] Profils de release : Il manque la configuration du profil de release (ex: `lto = true`, `opt-level = 3`) pour optimiser drastiquement les performances du binaire final en production.

**Suggestion d'ajout :**
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

---

## ✅ POINTS POSITIFS
- Les dépendances majeures comme `sqlx`, `lru`, `scraper` et `reqwest` ont été mises à jour récemment vers leurs versions patchées, éliminant les vulnérabilités critiques historiques (CVE-2024-XXXX, etc.).
- Utilisation systématique de `tls-rustls` plutôt que `native-tls` (OpenSSL), ce qui réduit la surface d'attaque et simplifie la compilation statique (pas de libssl système requise).

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 0 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 1 |
| ⚪ Qualité | 1 |

---
*Fichier suivant → `audit_02_nhtml_config.md`*
