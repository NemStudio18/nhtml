---
# 🔍 Audit — `src/compiler/mod.rs`
**Fichier d'audit** : `audit_14_compiler_mod.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_15_handler_table.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/compiler/mod.rs` |
| Rôle | Parsing du HTML source (`.nhtml`), extraction des comportements et génération du DOM sécurisé |
| Lignes | 567 |
| Langage | Rust |
| Score Sécurité | 🟢 9/10 |
| Score Performance | 🟡 8/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Aucune vulnérabilité n'a été détectée. L'implémentation est robuste.

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-14-001 — Risque de Stack Overflow sur DOM profond
- **Ligne(s)** : L.310 (`parse_element`)
- **Type** : Dépassement de pile (Recursion Limit)
- **Impact** : L'algorithme de parsing de l'arbre DOM est purement récursif. Si un attaquant ou un développeur fournit un fichier `.nhtml` contenant des dizaines de milliers de `<div>` imbriquées (ex: payload généré automatiquement), le thread Rust plantera avec un `Stack Overflow`, causant un DoS sur l'instance de compilation. 
- **Solution** : Cela n'est critique qu'en mode SaaS public (si NHTML permet l'upload de fichiers arbitraires). Pour limiter le risque, ajouter un compteur de profondeur `depth: u32` et rejeter le parsing si `depth > 2000`.

---

## ✅ POINTS POSITIFS
- **Protection XSS Serveur Stricte** : Lors de la reconstruction du HTML (`to_html`, L.134-158), l'utilisation stricte de `html_escape::encode_double_quoted_attribute` et `html_escape::encode_safe` prévient rigoureusement toute injection XSS au moment du Single-Page-Render initial.
- **Séparation des Attributs** : La liste `N_ATTRS` (L.179) garantit qu'aucun attribut système `n-*` n'est divulgué dans le DOM final rendu par le navigateur. C'est essentiel pour éviter les fuites de logique métier (handlers internes) côté client.

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 0 |
| 🟡 Moyennes | 0 |
| 🔵 Performance | 1 |
| ⚪ Qualité | 0 |

---
*Fichier suivant → `audit_15_handler_table.md`*
