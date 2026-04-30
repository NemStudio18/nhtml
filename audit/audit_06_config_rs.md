---
# 🔍 Audit — `src/config.rs`
**Fichier d'audit** : `audit_06_config_rs.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_07_socket_mod_part1.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/config.rs` |
| Rôle | Désérialisation et parsing du fichier TOML de configuration |
| Lignes | 82 |
| Langage | Rust |
| Score Sécurité | 🟢 9/10 |
| Score Performance | 🟢 10/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-06-001 — Pas d'enforcement de version TLS (Optionnel)
- **Ligne(s)** : L.34-39
- **Type** : Cryptographie faible (OWASP A02:2021)
- **Impact** : La configuration `TlsConfig` permet uniquement de déclarer si le TLS est activé et de fournir les chemins vers le certificat et la clé. Le Gateway pourrait négocier des versions TLS obsolètes (TLSv1.0 / TLSv1.1) si `rustls` modifie ses comportements par défaut ou selon le client. Il manque un champ pour forcer `min_tls_version = "1.2"`.

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [x] Désérialisation stricte : L'utilisation de `serde::Deserialize` est sûre. Aucune exécution de code ou désérialisation d'objet malicieux n'est possible via TOML dans ce contexte.
- [ ] Variables mortes : `#[allow(dead_code)]` est utilisé sur `dev` (L.9) et `http` (L.60). Il faudrait soit les retirer, soit les implémenter pleinement pour éviter l'accumulation de code zombie.

---

## ✅ POINTS POSITIFS
- Pas de crash (Panic) lors de la lecture d'un fichier invalide. La méthode `load()` utilise `.unwrap_or_default()` élégamment.
- Utilisation des `Option<T>` pour chaque section de la configuration, ce qui la rend parfaitement rétrocompatible.

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
*Fichier suivant → `audit_07_socket_mod_part1.md`*
