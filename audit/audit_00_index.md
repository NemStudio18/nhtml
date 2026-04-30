# 🛡️ Audit de Sécurité & Performance — NHTML Gateway
**Date** : 29 Avril 2026  
**Auditeur** : Claude Sonnet 4.6 (Expert Sécurité & Performance)  
**Méthodologie** : OWASP Top 10, SANS Top 25, CWE/CVE, Analyse O(n) & Profiling  

---

## 📂 Fichiers du projet analysés

Afin de garantir un audit impitoyable et exhaustif, les 22 fichiers sources et de configuration constituant le cœur de l'architecture NHTML (Gateway Rust, Client JS, SDK PHP et Infrastructure) ont été sélectionnés.

| # | Fichier source | Fichier d'audit | Rôle |
|---|---------------|-----------------|------|
| 01 | `Cargo.toml` | `audit_01_Cargo_toml.md` | Dépendances et configuration Rust |
| 02 | `nhtml.config.toml` | `audit_02_nhtml_config.md` | Configuration globale du Gateway |
| 03 | `.github/workflows/release.yml` | `audit_03_release_yml.md` | Pipeline CI/CD de déploiement |
| 04 | `assets/js/bridge.js` | `audit_04_bridge_js.md` | Client WebSocket et DOM Patcher JS |
| 05 | `src/main.rs` | `audit_05_main_rs.md` | Point d'entrée, initialisation, arguments |
| 06 | `src/config.rs` | `audit_06_config_rs.md` | Parseur et validation de configuration |
| 07 | `src/socket/mod.rs` | `audit_07_socket_mod_part1.md` | WebSocket, rate limiter, routing (Partie 1) |
| 08 | `src/socket/mod.rs` | `audit_08_socket_mod_part2.md` | Logique d'événements et broadcast (Partie 2) |
| 09 | `src/session.rs` | `audit_09_session_rs.md` | Gestion d'état et persistance base de données |
| 10 | `src/supervisor.rs` | `audit_10_supervisor_rs.md` | Process PHP auto-healing & client FPM |
| 11 | `src/decoder.rs` | `audit_11_decoder_rs.md` | Parsing du protocole binaire NBPS |
| 12 | `src/proto.rs` | `audit_12_proto_rs.md` | Constantes du protocole binaire |
| 13 | `src/compiler/mod.rs` | `audit_13_compiler_mod.md` | Parseur de templates NHTML vers HTML |
| 14 | `src/compiler/btree_builder.rs`| `audit_14_btree_builder.md` | Génération de l'arbre binaire DOM |
| 15 | `src/compiler/handler_table.rs`| `audit_15_handler_table.md` | Extraction et indexation des événements |
| 16 | `src/cluster.rs` | `audit_16_cluster_rs.md` | Synchronisation Redis inter-nœuds |
| 17 | `src/cli.rs` | `audit_17_cli_rs.md` | Interface ligne de commande (DevTools) |
| 18 | `src/watcher.rs` | `audit_18_watcher_rs.md` | Live-reload et filesystem watcher |
| 19 | `sdk/php/src/Nhtml.php` | `audit_19_sdk_php_nhtml.md` | SDK PHP Backend principal |
| 20 | `sdk/php/src/Patch.php` | `audit_20_sdk_php_patch.md` | Builder de patch DOM PHP |
| 21 | `examples/02-todo-list/app.php`| `audit_21_example_todo.md` | Exemple applicatif (Risques XSS/Injection) |
| 22 | `examples/05-chat/app.php` | `audit_22_example_chat.md` | Exemple collaboratif (Risques logique métier) |
| 23 | `audit_final_rapport.md` | — | Rapport consolidé et Plan d'Action |

---

## 📖 Comment lire cet audit
1. Commencer par ce fichier index pour comprendre le périmètre.
2. Lire chaque fichier `audit_NN_*.md` généré un par un, dans l'ordre chronologique.
3. Terminer par `audit_final_rapport.md` pour obtenir la note finale, l'analyse des chaînes d'attaques (Attack Chains) et le plan d'action priorisé par sprint.

## ⚠️ Légende des scores
🔴 **0–4** : Critique — Refonte ou correction immédiate requise  
🟠 **5–6** : Haut — Vulnérabilité majeure, correction à prioriser  
🟡 **7–8** : Moyen — Amélioration nécessaire, dette technique  
🟢 **9–10** : Bon — État de l'art, best practices respectées  

---
## 🚨 Alertes immédiates (Reconnaissance initiale)
- **Dépendances** : Le projet inclut `sqlx` avec des drivers optionnels (`mysql`, `postgres`) qui tirent transitivement `rsa 0.9` (vulnérable à Marvin Attack). Des mesures d'isolation réseau sont déjà recommandées.
- **Surface d'attaque** : L'exposition de l'interface `DevTools` (Port 8081) et le protocole WebSocket natif nécessiteront une vérification approfondie des contrôles d'accès et des buffers (risques d'OOM ou de DoS).
