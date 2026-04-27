# 🚀 Guide de Déploiement NHTML (Production)

Ce guide détaille comment déployer une application NHTML en production sur un serveur (ex: VPS Linux) derrière un reverse proxy standard (Nginx ou Apache).

En production, **le Gateway NHTML ne doit jamais être exposé directement sur Internet**. Il écoute sur `127.0.0.1` en interne, et c'est le serveur web (Nginx/Apache) qui s'occupe du SSL (HTTPS) et de router le trafic vers le Gateway.

## 🏗️ Architecture Cible

```text
Internet (Port 443 / HTTPS)
    │
    ▼
[ NGINX / APACHE ] ── (Reverse Proxy SSL)
    │
    ├── Requêtes HTTP (Assets, .nhtml)  ──► [ Gateway HTTP : 3000 ]
    └── Requêtes WS (NBPS Binaire)      ──► [ Gateway WS : 8080 ]
```
*Le port `8081` (DevTools) ne doit jamais être configuré dans le Reverse Proxy afin de rester privé et sécurisé.*

---

## 🟢 Déploiement avec Nginx (Recommandé)

Nginx gère très bien les WebSockets de manière native. Voici la configuration minimale `server {}` à placer dans `/etc/nginx/sites-available/mon-site`.

```nginx
server {
    listen 80;
    server_name nhtml.mon-domaine.com;

    # 1. Routing des WebSockets NHTML
    location /nhtml-ws {
        # Pointe vers le port WebSocket du Gateway
        proxy_pass http://127.0.0.1:8080;
        
        # Headers nécessaires pour maintenir la connexion WebSocket
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Sécurité
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # Important : Empêche la coupure prématurée des WebSockets inactifs
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    # 2. Routing HTTP (Pages .nhtml et assets)
    location / {
        # Pointe vers le serveur HTTP du Gateway
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 🔴 Déploiement avec Apache

Apache nécessite l'activation des modules de proxy. Sur Ubuntu/Debian, exécutez d'abord :
```bash
sudo a2enmod proxy proxy_http proxy_wstunnel
sudo systemctl restart apache2
```

Ensuite, configurez votre VirtualHost :

```apache
<VirtualHost *:80>
    ServerName nhtml.mon-domaine.com

    # 1. Routing des WebSockets NHTML
    RewriteEngine On
    RewriteCond %{HTTP:Upgrade} websocket [NC]
    RewriteCond %{HTTP:Connection} upgrade [NC]
    RewriteRule ^/nhtml-ws/?(.*) "ws://127.0.0.1:8080/$1" [P,L]

    # 2. Routing HTTP (Pages .nhtml et assets)
    ProxyPass /nhtml-ws/ !  # Demande à Apache de ne pas utiliser le proxy normal pour cette URL
    ProxyPass / http://127.0.0.1:3000/
    ProxyPassReverse / http://127.0.0.1:3000/

    # Sécurité des requêtes
    ProxyPreserveHost On
</VirtualHost>
```

---

## 🛡️ Sécurité & Accès aux DevTools (Port 8081)

Par défaut, les DevTools NHTML ne sont accessibles que sur `127.0.0.1`. C'est une mesure de sécurité cruciale pour éviter que n'importe qui puisse inspecter votre trafic ou rejouer des sessions en production.

Pour y accéder depuis votre machine locale sur un serveur distant, deux solutions s'offrent à vous :

### Solution 1 : Tunnel SSH (Recommandée)
C'est la méthode la plus sûre car elle ne nécessite aucune ouverture de port sur votre pare-feu et utilise le chiffrement de votre connexion SSH.

Lancez cette commande dans un terminal sur **votre machine locale** :
```bash
ssh -L 8081:127.0.0.1:8081 utilisateur@votre-serveur.com
```
Une fois connecté, ouvrez simplement [http://127.0.0.1:8081](http://127.0.0.1:8081) dans votre navigateur local.

### Solution 2 : Exposition directe avec Token
Si vous préférez un accès direct sans tunnel, vous pouvez demander au Gateway d'écouter sur toutes les interfaces, mais **vous devez impérativement utiliser un token de sécurité**.

Lancez le gateway avec ces paramètres :
```bash
./nhtml start --dev --devtools-host 0.0.0.0 --devtools-token VOTRE_CLE_SECRETE
```
L'URL d'accès sera alors : `http://votre-serveur.com:8081?token=VOTRE_CLE_SECRETE`

---

## ⚙️ Configuration du Gateway (`nhtml.config.toml`)

Pour correspondre à cette architecture, placez ce fichier `nhtml.config.toml` à la racine de votre application, à côté du binaire `nhtml` :

```toml
[ports]
ws = 8080
http = 3000
php = 8000
devtools = 8081

[dev]
auto_reload = false
```

## 🚀 Lancement en production

Ne lancez pas le Gateway directement dans le terminal (il s'arrêterait à la fermeture de la session). Utilisez **Systemd** ou un outil comme **PM2**.

### Exemple avec Systemd (`/etc/systemd/system/nhtml.service`) :
```ini
[Unit]
Description=NHTML Gateway Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/var/www/mon-app-nhtml
ExecStart=/var/www/mon-app-nhtml/nhtml start
Restart=on-failure

[Install]
WantedBy=multi-user.target
```
```bash
sudo systemctl enable nhtml
sudo systemctl start nhtml
```
