---
# 🔍 Audit — `src/main.rs`
**Fichier d'audit** : `audit_05_main_rs.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_06_config_rs.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `src/main.rs` |
| Rôle | Point d'entrée de l'application, instanciation des services |
| Lignes | 237 |
| Langage | Rust |
| Score Sécurité | 🟢 9/10 |
| Score Performance | 🟢 9/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-05-001 — Bind réseau permissif du Gateway
- **Ligne(s)** : L.211 (Appel à `socket::serve`)
- **Type** : Exposition réseau non sécurisée
- **Impact** : Bien que les DevTools soient correctement restreints sur `127.0.0.1` (L.177 et 215), le serveur web/websocket principal (géré via `socket::serve`) va probablement écouter sur `0.0.0.0` (toutes les interfaces réseau) en fonction de l'implémentation sous-jacente. L'absence de paramétrage explicite du host dans les CLI arguments (seulement le port est demandé L.24) force souvent un bind public.

**Code vulnérable** :
```rust
#[arg(short, long, default_value_t = 8080)]
port: u16,
```

**Code corrigé** :
```rust
#[arg(short, long, default_value_t = 8080)]
port: u16,

#[arg(long, default_value = "127.0.0.1")]
host: String, // Permettre explicitement de binder en localhost pour du dev
```

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-05-001 — Spawn de threads bloquants (Supervisor PHP)
- **Ligne(s)** : L.156-171
- **Type** : Thread exhaustion (Potentiel)
- **Impact** : Le supervisor démarre le serveur PHP dans un `tokio::spawn`. Si `supervisor::start_php_server` lance des processus lourds avec `std::process::Command` (qui est bloquant), cela risque de bloquer un thread du runtime Tokio asynchrone, impactant les performances globales du Gateway WebSocket.

**Avant** :
```rust
tokio::spawn(async move {
    let res = supervisor::start_php_server(...).await;
});
```

**Après** :
```rust
// Si le processus contient des appels bloquants lourds :
tokio::task::spawn_blocking(move || {
    // ... logic
});
// (Ou s'assurer que start_php_server utilise tokio::process::Command)
```

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [x] Initialisation propre : Le support des logs structurés (JSON) via `tracing_subscriber::fmt().json()` pour Datadog/ELK est une excellente pratique.
- [x] Résolution de l'incident CLI : La base de données est maintenant correctement lue via variable d'environnement (`NHTML_DB_URI`), ce qui corrige la vulnérabilité historique d'exposition via `ps aux`.

---

## ✅ POINTS POSITIFS
- Protection DevTools : Le Dashboard DevTools (Port 8081/8082) est hardcodé pour n'écouter que sur `127.0.0.1` (L.177), ce qui empêche une compromission critique via l'exposition publique de l'administration.
- Utilisation des "Channels" asynchrones (`broadcast::channel`) dimensionnés de manière protectrice (10000 events max) pour prévenir l'OOM (Out Of Memory) en cas de surcharge.

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
*Fichier suivant → `audit_06_config_rs.md`*
