# ⚙️ NHTML Configuration Reference (v0.7.3)

*(Scroll down for the French version / La version française se trouve plus bas)*

---

## 🇬🇧 English Reference

This document describes all available options in the `nhtml.config.toml` file.

### 🔌 `[ports]`
Network ports for the different services.
*   **`ws`** (u16): WebSocket Gateway port (Default: `8080`).
*   **`php`** (u16): Internal PHP dev server port (Default: `8000`).
*   **`devtools`** (u16): NHTML DevTools interface port (Default: `8081`).
*   **`http`** (u16): Standalone HTTP port (reserved for future use).

### 🚀 `[fastcgi]`
High-performance backend configuration for PHP-FPM.
*   **`address`** (string): Address of a single PHP-FPM pool (e.g., `127.0.0.1:9000` or `unix:/var/run/php-fpm.sock`).
*   **`addresses`** (array of strings): List of addresses for load balancing across multiple PHP-FPM pools.
*   **`strategy`** (string): Load balancing strategy if multiple addresses are used (`round-robin` or `least-conn`).
*   **`timeout_ms`** (u64): Socket timeout for FPM requests in milliseconds (Default: `5000`).

### 🛡️ `[security]`
Security and traffic control settings.
*   **`allowed_origins`** (array of strings): List of allowed origins for WebSocket connections (CSWH protection). (Default: `["*"]` in dev).

#### `[security.tls]`
*   **`enabled`** (bool): Enables Native TLS (HTTPS/WSS) on the Gateway.
*   **`cert`** (string): Path to the SSL certificate (`.pem`).
*   **`key`** (string): Path to the SSL private key (`.pem`).
*   **`min_version`** (string): Minimum TLS version accepted (e.g., `"1.3"`).

#### `[security.rate_limit]`
*   **`events_per_sec`** (u32): Maximum NBPS events allowed per IP per second to prevent DoS. (Default: `10`).

### 🗄️ `[database]`
Internal state and session storage.
*   **`driver`** (string): Database driver (`sqlite`, `mysql`, `postgres`).
*   **`uri`** (string): Connection URI (e.g., `nhtml_sessions.db` or `mysql://user:pass@host/db`).

---

## 🇫🇷 Référence en Français

Ce document décrit toutes les options disponibles dans le fichier `nhtml.config.toml`.

### 🔌 `[ports]`
Ports réseau pour les différents services.
*   **`ws`** (u16) : Port du Gateway WebSocket (Défaut : `8080`).
*   **`php`** (u16) : Port du serveur de développement PHP interne (Défaut : `8000`).
*   **`devtools`** (u16) : Port de l'interface NHTML DevTools (Défaut : `8081`).
*   **`http`** (u16) : Port HTTP dédié (réservé pour usage futur).

### 🚀 `[fastcgi]`
Configuration du backend haute performance PHP-FPM.
*   **`address`** (chaîne) : Adresse d'un pool PHP-FPM unique (ex: `127.0.0.1:9000` ou `unix:/var/run/php-fpm.sock`).
*   **`addresses`** (tableau de chaînes) : Liste d'adresses pour la répartition de charge (Load Balancing) sur plusieurs pools.
*   **`strategy`** (chaîne) : Stratégie de load balancing si `addresses` est utilisé (`round-robin` ou `least-conn`).
*   **`timeout_ms`** (u64) : Délai d'attente maximum pour les requêtes FPM en millisecondes (Défaut : `5000`).

### 🛡️ `[security]`
Paramètres de sécurité et de contrôle du trafic.
*   **`allowed_origins`** (tableau de chaînes) : Liste des origines autorisées pour les connexions WebSocket (Protection CSWH).

#### `[security.tls]`
*   **`enabled`** (booléen) : Active le TLS natif (HTTPS/WSS) sur le Gateway.
*   **`cert`** (chaîne) : Chemin vers le certificat SSL (`.pem`).
*   **`key`** (chaîne) : Chemin vers la clé privée SSL (`.pem`).
*   **`min_version`** (chaîne) : Version TLS minimale acceptée (ex: `"1.3"`).

#### `[security.rate_limit]`
*   **`events_per_sec`** (u32) : Nombre maximum d'événements autorisés par IP et par seconde pour prévenir les dénis de service (DoS).

### 🗄️ `[database]`
Stockage interne pour l'état et les sessions.
*   **`driver`** (chaîne) : Pilote de base de données (`sqlite`, `mysql`, `postgres`).
*   **`uri`** (chaîne) : URI de connexion (ex: `nhtml_sessions.db` ou `mysql://user:pass@host/db`).

---

## Example / Exemple `nhtml.config.toml`
```toml
[ports]
ws = 8080
php = 8000
devtools = 8081

[fastcgi]
# Single pool / Pool unique
address = "127.0.0.1:9000"
timeout_ms = 3000

# Load balancing (Optional)
# addresses = ["127.0.0.1:9000", "127.0.0.1:9001"]
# strategy = "round-robin"

[security]
allowed_origins = ["https://my-app.com", "https://admin.my-app.com"]

[security.tls]
enabled = true
cert = "/etc/letsencrypt/live/my-app.com/fullchain.pem"
key = "/etc/letsencrypt/live/my-app.com/privkey.pem"
min_version = "1.3"

[security.rate_limit]
events_per_sec = 20

[database]
driver = "sqlite"
uri = "nhtml_sessions.db"
```
