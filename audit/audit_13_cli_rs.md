---
# 🔍 Audit — `src/cli.rs`
**Fichier d'audit** : `audit_13_cli_rs.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_14_compiler.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/cli.rs` |
| Rôle | Outils en ligne de commande (DevTools, Build, Inspect, Scaffold) |
| Lignes | 651 |
| Langage | Rust |
| Score Sécurité | 🟡 8/10 |
| Score Performance | 🟢 9/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
### MED-13-001 — Utilisation de `npx localtunnel` (Dépendance externe dynamique)
- **Ligne(s)** : L.576
- **Type** : Supply Chain / Exécution non maîtrisée
- **Impact** : La commande `nhtml share` exécute `npx localtunnel`. `npx` télécharge et exécute silencieusement le paquet npm s'il n'est pas présent. Si ce paquet npm venait à être compromis à l'avenir, son exécution donnerait un accès total à la machine du développeur.

**Recommandation** : Restreindre cette option ou afficher un avertissement de sécurité avant le téléchargement implicite via NPM.

---

## ⚪ QUALITÉ & MAINTENABILITÉ
### QUAL-13-001 — Commande `xcopy` non portable (Windows uniquement)
- **Ligne(s)** : L.642
- **Type** : Incompatibilité cross-platform
- **Impact** : Lors d'un `nhtml build`, si le dossier `assets` existe, l'outil invoque `xcopy` via le shell. Cette commande échouera lamentablement sur macOS et Linux, cassant la pipeline de build pour la moitié des utilisateurs.

**Avant** :
```rust
let _ = std::process::Command::new("xcopy")
    .args(["/E", "/I", "/Y", "assets", &format!("{}\\assets", output_dir)])
    .status();
```

**Après** :
```rust
// Utiliser la crate `fs_extra` ou implémenter une copie récursive en pur Rust
// pour garantir la compatibilité multi-plateformes.
```

---

## ✅ POINTS POSITIFS
- L'administration DevTools vérifie proprement le `token` HTTP et WebSocket (L.134 et L.150).
- Pas d'exposition dangereuse au réseau public de la CLI, excepté via le tunnel explicite autorisé par l'utilisateur.

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
*Fichier suivant → `audit_14_compiler.md` (Dossier)*
