# 🚀 NHTML Deployment Guide (Production)

This guide details how to deploy an NHTML application in production on a server (e.g., Linux VPS) behind a standard reverse proxy (Nginx or Apache).

In production, **the NHTML Gateway should never be directly exposed to the Internet**. It listens on `127.0.0.1` internally, and the web server (Nginx/Apache) handles SSL (HTTPS) and routing traffic to the Gateway.

## 🏗️ Target Architecture

```text
Internet (Port 443 / HTTPS)
    │
    ▼
[ NGINX / APACHE ] ── (SSL Reverse Proxy)
    │
    ├── HTTP Requests (Assets, .nhtml)  ──► [ Gateway HTTP : 3000 ]
    └── WS Requests (Binary NBPS)       ──► [ Gateway WS : 8080 ]
```
*Port `8081` (DevTools) should never be configured in the Reverse Proxy to remain private and secure.*

---

## 🟢 Deployment with Nginx (Recommended)

Nginx handles WebSockets natively very well. Here is the minimum `server {}` configuration to place in `/etc/nginx/sites-available/my-site`.

```nginx
server {
    listen 80;
    server_name nhtml.my-domain.com;

    # 1. Routing NHTML WebSockets
    location /nhtml-ws {
        # Points to the Gateway's WebSocket port
        proxy_pass http://127.0.0.1:8080;
        
        # Required headers to maintain the WebSocket connection
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Security
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # Important: Prevents premature timeout for inactive WebSockets
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    # 2. HTTP Routing (.nhtml pages and assets)
    location / {
        # Points to the Gateway's HTTP server
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 🔴 Deployment with Apache

Apache requires enabling proxy modules. On Ubuntu/Debian, first run:
```bash
sudo a2enmod proxy proxy_http proxy_wstunnel
sudo systemctl restart apache2
```

Then, configure your VirtualHost:

```apache
<VirtualHost *:80>
    ServerName nhtml.my-domain.com

    # 1. Routing NHTML WebSockets
    RewriteEngine On
    RewriteCond %{HTTP:Upgrade} websocket [NC]
    RewriteCond %{HTTP:Connection} upgrade [NC]
    RewriteRule ^/nhtml-ws/?(.*) "ws://127.0.0.1:8080/$1" [P,L]

    # 2. HTTP Routing (.nhtml pages and assets)
    ProxyPass /nhtml-ws/ !  # Tell Apache not to use the normal proxy for this URL
    ProxyPass / http://127.0.0.1:3000/
    ProxyPassReverse / http://127.0.0.1:3000/

    # Security for requests
    ProxyPreserveHost On
</VirtualHost>
```

---

## 🛡️ Security & DevTools Access (Port 8081)

By default, NHTML DevTools are only accessible on `127.0.0.1`. This is a crucial security measure to prevent anyone from inspecting your traffic or replaying sessions in production.

To access them from your local machine on a remote server, two solutions are available:

### Solution 1: SSH Tunnel (Recommended)
This is the safest method as it requires no port openings on your firewall and uses your SSH connection's encryption.

Run this command in a terminal on **your local machine**:
```bash
ssh -L 8081:127.0.0.1:8081 user@your-server.com
```
Once connected, simply open [http://127.0.0.1:8081](http://127.0.0.1:8081) in your local browser.

### Solution 2: Direct Exposure with Token
If you prefer direct access without a tunnel, you can tell the Gateway to listen on all interfaces, but **you must use a security token**.

Launch the gateway with these parameters:
```bash
./nhtml start --dev --devtools-host 0.0.0.0 --devtools-token YOUR_SECRET_KEY
```
The access URL will then be: `http://your-server.com:8081?token=YOUR_SECRET_KEY`

---

## ⚙️ Gateway Configuration (`nhtml.config.toml`)

To match this architecture, place this `nhtml.config.toml` file at the root of your application, next to the `nhtml` binary:

```toml
[ports]
ws = 8080
http = 3000
php = 8000
devtools = 8081

[dev]
auto_reload = false
```

## 🚀 Launching in Production

Do not launch the Gateway directly in the terminal (it will stop when the session ends). Use **Systemd** or a tool like **PM2**.

### Systemd Example (`/etc/systemd/system/nhtml.service`):
```ini
[Unit]
Description=NHTML Gateway Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/var/www/my-nhtml-app
ExecStart=/var/www/my-nhtml-app/nhtml start
Restart=on-failure

[Install]
WantedBy=multi-user.target
```
```bash
sudo systemctl enable nhtml
sudo systemctl start nhtml
```
