# 🛰️ Référence Technique NHTML (v0.7.4)
**Document de référence pour l'architecture industrialisée.**

---

## 1. Vue d'ensemble
NHTML est un framework de développement web "Server-Driven" ultra-performant. Il transforme le navigateur en un moteur de rendu binaire piloté par un backend.

- **Gateway** : Serveur Rust (Tokio) gérant le transport binaire, la sécurité HMAC et le multiplexage.
- **Protocol** : NBPS (Native-HTML Binary Protocol) avec compression Zstd adaptative.
- **Backend** : Vos applications PHP (CGI ou FastCGI/FPM).

---

## 2. Architecture Générale (v0.7.4)
L'architecture repose sur un **Gateway Orchestrateur** agissant comme un pont binaire bidirectionnel sécurisé.

```mermaid
graph TD
    Browser[Navigateur (bridge.js)] <-->|NBPS Binaire + HMAC| Gateway[Gateway Rust]
    
    subgraph Backend
        Gateway <-->|FastCGI / TCP| FPM[Pool PHP-FPM]
        Gateway <-->|CGI / Stdout| PHP[App PHP CLI]
    end
    
    Gateway -->|Circuit Breaker| FPM
    Gateway <-->|Delta Sync| Browser
    Gateway -->|Monitoring| DevTools[Dashboard DevTools 8081]
    Gateway <-->|SQLite WAL| DB_Sessions[nhtml_sessions.db]
```

---

## 3. Communication & Résilience (v0.7.4)
- **FastCGI Load Balancing** : Dispatching intelligent (Round-Robin/Least-Connections) vers les pools PHP-FPM.
- **Circuit Breaker** : Coupure automatique du trafic vers les backends instables pour prévenir les pannes en cascade.
- **Delta Sync** : Récupération intelligente de l'état après déconnexion via rejeu de patchs (au lieu d'un snapshot complet).

---

## 4. Collaboration Temps Réel (Broadcasting)
Le Gateway agit comme un serveur de messagerie binaire. 
1. Une session envoie un événement.
2. Le backend traite et retourne des mutations pour l'envoyeur ET des instructions de diffusion.
3. Le Gateway route instantanément les paquets aux autres clients concernés via le bus de données interne.

---

## 5. Sécurité Industrielle
Chaque interaction est protégée par :
- **HMAC-SHA256** : Garantit l'origine et l'intégrité des paquets.
- **Sequence ID** : Empêche toute attaque par rejeu.
- **CSWH Protection** : Validation de l'Origin des WebSockets.

---

## 6. Glossaire
- **NID** : Identifiant textuel (string) mappé dynamiquement à un ID binaire u16.
- **FPM (FastCGI Process Manager)** : Système de gestion de processus PHP pour la production.
- **Delta History** : Journal transactionnel des mutations permettant le Delta Sync.
- **WAL (Write-Ahead Logging)** : Mode de journalisation SQLite optimisé pour la concurrence.

