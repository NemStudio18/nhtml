---
# 🔍 Audit — `src/socket/mod.rs` (Partie 2: Gestion des événements & FPM)
**Fichier d'audit** : `audit_08_socket_mod_part2.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_09_session_rs.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/socket/mod.rs` (Reste du fichier, Parsing et Routage PHP) |
| Rôle | Interfaçage avec le backend PHP, Broadcast de paquets |
| Langage | Rust |
| Score Sécurité | 🟡 7/10 |
| Score Performance | 🟡 8/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-08-001 — Gestion bloquante du FastCGI Pool
- **Ligne(s)** : ~L.152 (`FpmPool::acquire`)
- **Type** : Déni de Service (DoS) par épuisement de pool
- **Impact** : Si PHP est lent à répondre, la limite de connexions `max_size` (100) est rapidement atteinte. Le code renvoie alors immédiatement une erreur de socket `FpmPool saturé`. Sous une attaque DoS applicative mineure, tous les clients légitimes recevront des erreurs au lieu d'être mis en file d'attente avec timeout.

**Avant** :
```rust
if curr >= self.max_size {
    return Err(GatewayError::SocketError("FpmPool saturé"));
}
```

**Après** :
```rust
// Utiliser un tokio::sync::Semaphore pour mettre les requêtes en file d'attente
// (avec un timeout asynchrone) plutôt que de rejeter brutalement.
```

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-08-001 — Compilation JIT synchrone des fichiers NHTML
- **Ligne(s)** : L.660-681
- **Type** : Blocage du runtime
- **Impact** : La compilation `.nhtml` via `NhtmlCompiler::compile(&source)` s'exécute à l'intérieur de la méthode asynchrone `handle_connection_axum` sans être déléguée à un pool de threads bloquants. Si le fichier source est lourd, cela "stalle" (bloque) le thread de l'exécuteur Tokio, pénalisant tous les autres WebSockets hébergés par ce même thread.

**Solution recommandée** :
Envelopper l'appel dans un `tokio::task::spawn_blocking` :
```rust
let res = Arc::new(tokio::task::spawn_blocking(move || {
    NhtmlCompiler::compile(&source)
}).await.unwrap());
```

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [ ] Taille du B-TREE : L'envoi immédiat du B-TREE entier au démarrage (L.748) peut saturer la bande passante initiale sur de très grosses applications. La compression `Zstd` implémentée récemment atténue grandement ce problème.

---

## ✅ POINTS POSITIFS
- Le "Load Balancer" FastCGI (`FpmLoadBalancer`, L.61) est très bien implémenté avec des "Health Checks" périodiques. Cela évite d'envoyer des événements à des processus PHP morts.
- La validation des requêtes Unix Socket vs TCP est très propre et empêche d'exploiter les chemins Unix Socket malicieusement sur un système non-Unix.

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
*Fichier suivant → `audit_09_session_rs.md`*
