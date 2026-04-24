# 🏛️ Architecture de Déploiement NHTML

Le framework NHTML a été conçu avec un protocole binaire (NBPS) très modulaire, ce qui permet à l'écosystème de s'adapter à plusieurs stratégies de déploiement en production, selon les besoins en performances et l'infrastructure disponible.

---

## 1. Mode "Serveur Dédié" (Nginx + Gateway Rust)
**Cas d'usage :** Applications à fort trafic, nécessitant du vrai temps réel (Jeux multijoueurs, Dashboards financiers, Chats).
**Infrastructure requise :** VPS ou Serveur Dédié (accès CLI complet).

* **Comment ça marche ?**
  L'utilisateur installe le binaire compilé `nhtml` (ex: `nhtml.exe` ou `nhtml-linux`) sur son serveur et le lance en tâche de fond (`nhtml start`). Un serveur web classique (comme **Nginx** ou **Apache**) est placé devant sur le port 80/443 pour gérer les certificats SSL.
* **Le lien avec Nginx :**
  Nginx est configuré en mode **Reverse Proxy**. Il sert directement les fichiers statiques (images, CSS), mais dès qu'une connexion WebSocket (`/ws`) arrive, il la redirige vers le processus Rust tournant sur le port local (ex: 8080).
* **Avantage :** Vitesse pure. Le binaire Rust maintient les WebSockets ouverts de manière asynchrone et discute avec PHP (via FastCGI ou Swoole).

---

## 2. Mode "Hébergement Mutualisé" (PHP Fallback)
**Cas d'usage :** Sites vitrines, blogs, petites boutiques, applications standard.
**Infrastructure requise :** Un simple hébergement mutualisé OVH/Hostinger (accès FTP uniquement, sans accès CLI).

* **Comment ça marche ?**
  Puisque le développeur ne peut pas lancer de binaire Rust ou ouvrir de ports WebSocket, NHTML dégrade gracieusement son protocole.
* **Le lien avec Apache/PHP :**
  Le fichier `bridge.js` (le Polyfill côté client) tente d'ouvrir un WebSocket. S'il échoue, il bascule automatiquement en mode **HTTP/AJAX**. Au lieu d'envoyer un paquet binaire `EVENT` par un tunnel ouvert, il va faire un simple `POST /nhtml-endpoint.php` avec le payload binaire.
* **Le traitement :**
  Le backend PHP reçoit la requête HTTP, traite la logique métier, et renvoie un paquet binaire `PATCH` en guise de réponse HTTP. Le Polyfill met ensuite à jour le DOM.
* **Avantage :** 100% universel. Pas de configuration serveur requise. Tu bénéficies de la réactivité de NHTML sans changer d'hébergeur.

---

## 3. Mode "Statique & Serverless" (WASM In-Browser)
**Cas d'usage :** Projets gratuits, PoC, applications sans base de données lourde.
**Infrastructure requise :** Hébergement statique (GitHub Pages, Vercel, Netlify, Amazon S3).

* **Comment ça marche ?**
  C'est la vision finale (Phase 3) du framework. Le code de backend (moteur NHTML et/ou interpréteur PHP) est compilé en **WebAssembly (WASM)**.
* **Le fonctionnement :**
  Le visiteur arrive sur un site 100% statique (fichiers HTML et JS seulement). Le Polyfill JavaScript télécharge le moteur WASM en arrière-plan et l'exécute **dans le navigateur du visiteur**.
  Tes fichiers `app.php` sont téléchargés statiquement. Quand l'utilisateur clique, le Polyfill ne fait **aucune requête réseau** : il passe l'événement au moteur WASM local qui exécute le code PHP et génère le `PATCH` binaire instantanément.
* **Avantage :** Hébergement à 0€. Vitesse infinie (zéro latence réseau). Confidentialité maximale (les calculs sont faits chez l'utilisateur).

---

## 📍 Où en sommes-nous ?

- **Le Mode "Serveur Dédié" (Gateway Rust) :** 🟢 Opérationnel ! C'est ce que nous développons actuellement. Le Gateway fait le pont parfait entre le JS et le PHP.
- **Le Mode "Hébergement Mutualisé" (PHP Fallback) :** 🟡 En cours. Le SDK PHP binaire (v0.2.2) est prêt à formater les paquets. Il nous reste à intégrer la bascule automatique (WebSocket -> HTTP) dans `bridge.js`.
- **Le Mode "Statique & Serverless" (WASM) :** ⚪ À l'étude. Des projets comme `php-wasm` prouvent que c'est possible, mais cela demandera une adaptation complète de l'interpréteur NHTML.
