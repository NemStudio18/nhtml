---
# 🔍 Audit — `src/compiler/btree_builder.rs`
**Fichier d'audit** : `audit_16_btree_builder.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_17_proto_rs.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/compiler/btree_builder.rs` |
| Rôle | Sérialisation de l'arbre DOM vers le format binaire B-TREE |
| Lignes | 78 |
| Langage | Rust |
| Score Sécurité | 🟡 6/10 |
| Score Performance | 🟢 9/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger dans les prochains mois

### MED-16-001 — Troncature silencieuse des tailles de chaînes (Cast u8 / u16)
- **Ligne(s)** : L.26, L.46, L.52
- **Type** : Troncature d'Entier (Integer Truncation / Overflow)
- **Impact** : Lors de la sérialisation binaire, la taille des attributs ou des balises est castée directement via `as u8` ou `as u16`.
  Par exemple : `buf.push(tag.len() as u8);`. Si un développeur a défini un tag custom très long (ex: 260 caractères) ou un attribut dont la valeur dépasse 65535 octets, la taille stockée `(len & 0xFF)` sera erronée, ce qui corrompra tout le reste du flux binaire B-TREE silencieusement.
  
**Code vulnérable** :
```rust
buf.push(tag.len() as u8); // Déborde si > 255
buf.push((vb.len() >> 8) as u8); // Déborde si > 65535
```

**Code corrigé** :
```rust
let tag_len = std::cmp::min(tag.len(), 255);
buf.push(tag_len as u8);
// Et de même pour `u16` sur les attributs textuels.
```

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-16-001 — Allocation Vectorielle (Vec::extend)
- **Ligne(s)** : L.10
- **Type** : Optimisation
- **Impact** : `Vec::new()` crée un vecteur avec une capacité de 0. Chaque appel à `push` ou `extend_from_slice` va déclencher de multiples réallocations (et copies mémoires) lors de la construction du B-TREE.
  Il est recommandé de pré-allouer la taille avec `Vec::with_capacity(1024)` ou plus pour réduire la fragmentation mémoire.

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 0 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 1 |
| ⚪ Qualité | 0 |

---
*Fichier suivant → `audit_17_proto_rs.md`*
