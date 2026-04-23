# 🚀 NHTML Direct Binary Installation (Standalone)

Run the NHTML Gateway as a high-performance standalone binary. This is the recommended way to deploy NHTML in production.

## Step 1: Download & Install
Download the latest `gateway` binary for your OS (Windows/Linux/MacOS).

```bash
# Example for Linux
chmod +x gateway
sudo mv gateway /usr/local/bin/nhtml
```

## Step 2: Running the Gateway
The Gateway acts as a Supervisor. It handles connections, persistence, and can even launch your PHP background processes.

### Production Mode
```bash
nhtml start --port 8080 --db ./sessions.db
```

### Development Mode (with Watcher)
```bash
nhtml dev --watch ./templates
```

## Step 3: Reverse Proxy (Optional)
If you are using Nginx, you can proxy the WebSocket traffic to the Gateway.

```nginx
location /gateway {
    proxy_pass http://localhost:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "Upgrade";
}
```

---

# 🇫🇷 Installation Directe du Binaire NHTML (v0.2.2)

Exécutez le Gateway NHTML comme un service binaire haute performance. C'est la méthode recommandée pour la production.

## Étape 1 : Installation
Placez le binaire `gateway` dans votre PATH.

## Étape 2 : Lancement
Le Gateway est autonome. Il gère les sockets, la base de données SQLite et la sécurité P3.

- **Mode Production** : `nhtml start`
- **Mode Développeur** : `nhtml dev` (Active le rechargement automatique et les logs détaillés).

## Étape 3 : Diagnostic Intégré
Utilisez les outils fournis pour surveiller votre instance :
- `nhtml db-dump` : Voir l'état des sessions.
- `nhtml inspect <hex>` : Analyser un paquet binaire suspect.

---
**Status :** Industriel. Cette méthode offre les meilleures performances et une isolation complète de l'application.
