# 🛰️ Référence Technique NHTML (v0.6.0)
**Document de référence pour l'architecture industrialisée.**

---

## 1. Vue d'ensemble
NHTML est un framework de développement web "Server-Driven" ultra-performant. Il transforme le navigateur en un moteur de rendu binaire piloté par un backend.

- **Gateway** : Serveur Rust (Tokio) gérant le transport binaire, la sécurité HMAC et le multiplexage.
- **Protocol** : NBPS (Native-HTML Binary Protocol) avec compression Zstd native.
- **Backend** : Vos applications PHP (CGI ou FastCGI/FPM).

---

## 2. Architecture Générale (v0.6.0)
L'architecture repose sur un **Gateway Orchestrateur** agissant comme un pont binaire bidirectionnel sécurisé.

```mermaid
graph TD
    Browser[Navigateur (bridge.js)] <-->|NBPS Binaire + HMAC| Gateway[Gateway Rust]
    
    subgraph Backend
        Gateway <-->|FastCGI / TCP| FPM[Pool PHP-FPM]
        Gateway <-->|CGI / Stdout| PHP[App PHP CLI]
    end
    
    Gateway -->|Broadcast| Others[Autres Sessions]
    Gateway -->|Monitoring| DevTools[Dashboard DevTools 8081]
    Gateway <-->|SQLite| DB_Sessions[nhtml_sessions.db]
```

---

## 3. Communication Haute Performance
NHTML v0.6.0 introduit le support natif de **FastCGI**. 
Au lieu de lancer un processus PHP pour chaque clic, le Gateway maintient des sockets ouverts vers un pool PHP-FPM, réduisant la latence à < 5ms.

---

## 4. Collaboration Temps Réel (Broadcasting)
Le Gateway agit comme un serveur de messagerie binaire. 
1. Une session envoie un événement.
2. Le backend traite et retourne des mutations pour l'envoyeur ET des instructions de diffusion.
3. Le Gateway route instantanément les paquets aux autres clients concernés.

---

## 5. Sécurité Industrielle (v0.5.0)
Chaque interaction est protégée par :
- **HMAC-SHA256** : Garantit l'origine et l'intégrité des paquets.
- **Sequence ID** : Empêche toute attaque par rejeu.

---

## 6. Glossaire
- **NID** : Identifiant textuel (string) mappé dynamiquement à un ID binaire u16.
- **FPM (FastCGI Process Manager)** : Système de gestion de processus PHP pour la production.
- **Zero-JS** : Concept où aucun code JavaScript métier n'est écrit par le développeur.
