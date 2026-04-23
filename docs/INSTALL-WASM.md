# 🦀 NHTML Browser Polyfill (WASM) Guide v0.2.2

In the industrial architecture, the browser acts as a **lightweight terminal**. The WebAssembly Polyfill is the engine that connects to the Gateway and renders binary updates.

## Prerequisites
- NHTML Gateway running on your server (Port 8080 by default).
- Browsers with WASM support (98% of modern traffic).

## Step 1: Include the Polyfill
Add the bootstrapper to your main HTML page. This will automatically load the WASM engine.

```html
<script src="dist/nhtml-polyfill.js"></script>
```

## Step 2: Initialize Connection
The Polyfill needs to know where the Gateway is. It uses WebSocket to receive NBPS binary packets.

```javascript
// Automatically connect to the local gateway
Nhtml.connect("ws://your-server:8080/gateway");

// The Polyfill now handles HELLO, PATCH, and EVENT packets automatically.
```

## Step 3: Interactive Elements
Elements with a `data-n-id` (NodeID) are automatically tracked by the Polyfill. When a user interacts with them, an **EVENT** packet is sent to the Gateway.

---

# 🇫🇷 Guide Polyfill Navigateur NHTML (WASM) v0.2.2

Dans l'architecture industrielle, le navigateur est un **terminal léger**. Le Polyfill WebAssembly est le moteur qui se connecte au Gateway et exécute les ordres binaires.

## Étape 1 : Inclure le Polyfill
Ajoutez le script de démarrage à votre page HTML.

```html
<script src="dist/nhtml-polyfill.js"></script>
```

## Étape 2 : Initialisation
Le Polyfill se connecte au Gateway via WebSocket pour recevoir le flux NBPS.

```javascript
Nhtml.connect("ws://votre-serveur:8080/gateway");
```

## Étape 3 : Fonctionnement
- **HELLO** : Le Polyfill s'identifie auprès du serveur.
- **PATCH** : Le moteur WASM applique les mutations DOM binaires à une vitesse native.
- **EVENT** : Les clics et saisies sont capturés et renvoyés au Gateway de manière sécurisée.

---
**Status :** Industriel. Cette méthode garantit une latence minimale et une sécurité maximale (P3).
