---
# 🔍 Audit — `src/session.rs`
**Fichier d'audit** : `audit_09_session_rs.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_10_decoder_rs.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/session.rs` |
| Rôle | Gestion de la persistance des sessions et de l'état du DOM (SQLite/MySQL/PostgreSQL) |
| Lignes | 270 |
| Langage | Rust (sqlx) |
| Score Sécurité | 🟡 8/10 |
| Score Performance | 🟡 7/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-09-001 — Incohérence de données lors du nettoyage TTL (Pas de transaction)
- **Ligne(s)** : L.256-266 (`cleanup_expired_sessions`)
- **Type** : Perte d'intégrité des données
- **Impact** : Lors de la suppression d'une session expirée, le Gateway exécute 6 requêtes `DELETE` consécutives sur différentes tables. Ces requêtes ne sont pas encapsulées dans une transaction (`BEGIN TRANSACTION ... COMMIT`). Si la base de données subit une micro-coupure ou si le processus crash au milieu, la base se retrouve dans un état orphelin (zombie data) ce qui peut causer des comportements indéfinis.

**Code vulnérable** :
```rust
for sid in expired {
    sqlx::query("DELETE FROM nodes...").execute(&self.pool).await.ok();
    // ...
    sqlx::query("DELETE FROM sessions...").execute(&self.pool).await.ok();
}
```

**Code corrigé** :
```rust
for sid in expired {
    let mut tx = self.pool.begin().await?;
    sqlx::query("DELETE FROM nodes...").execute(&mut tx).await?;
    // ...
    tx.commit().await?;
}
```

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-09-001 — Requête "DELETE" en boucle (N+1 Queries)
- **Ligne(s)** : L.256-266
- **Type** : Mauvaise pratique ORM/DB (N+1)
- **Impact** : Le nettoyeur TTL itère sur la liste des `session_id` expirés et effectue les 6 suppressions UNE par UNE. Si 5000 sessions ont expiré, cela engendre 30 000 requêtes vers la base de données d'un coup, ce qui paralysera temporairement PostgreSQL/MySQL ou causera un lock SQLite (`database is locked`).

**Solution recommandée** :
Effectuer un `DELETE WHERE session_id IN (...)` ou utiliser des requêtes par lots (batch) via `QueryBuilder` de `sqlx`. Encore mieux, définir les clés étrangères (Foreign Keys) en `ON DELETE CASCADE` dans les `CREATE TABLE` !

---

## ✅ POINTS POSITIFS
- **Requêtes Préparées strictes** : **Toutes** les requêtes SQL (sans exception) utilisent la méthode `.bind()`. Les injections SQL (OWASP A03) sont rigoureusement impossibles.
- **Support Universel `AnyPool`** : Excellente architecture. Gérer finement `max_conns = 1` pour SQLite sans WAL (L.17) évite d'innombrables erreurs de concurrence en production pour les petits déploiements.
- **Crypto-Secure RNG** : La génération du secret pour HMAC (L.121) utilise bien la crate `getrandom`, garantissant de l'entropie système sûre.

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
*Fichier suivant → `audit_10_decoder_rs.md`*
