# ⚙️ NHTML Configuration Reference (v0.7.1)

This document describes all available options in `nhtml.config.toml`.

---

## 🔌 `[ports]`
Network ports for the different services.
*   **`ws`** (u16): WebSocket Gateway port (Default: `8080`).
*   **`php`** (u16): Internal PHP dev server port (Default: `8000`).
*   **`devtools`** (u16): NHTML DevTools interface port (Default: `8081`).

---

## 🚀 `[fastcgi]`
High-performance backend configuration.
*   **`address`** (string): Address of the PHP-FPM pool (e.g., `127.0.0.1:9000` or `unix:/var/run/php-fpm.sock`).
*   **`timeout_ms`** (u32): Socket timeout for FPM requests (Default: `5000`).

---

## 🛡️ `[security]`
Security and traffic control settings.

### `[security.tls]`
*   **`enabled`** (bool): Enables Native TLS (HTTPS/WSS).
*   **`cert`** (string): Path to the SSL certificate (`.pem`).
*   **`key`** (string): Path to the SSL private key (`.pem`).

### `[security.rate_limit]`
*   **`events_per_sec`** (u32): Maximum events allowed per IP per second (Default: `10`).

### `[security.cors]`
*   **`allowed_origins`** (array of strings): List of domains allowed to connect to the Gateway. (Default: `*` in dev, strict in prod).

---

## 🗄️ `[database]`
Internal state and session storage.
*   **`driver`** (string): Database driver (`sqlite`).
*   **`uri`** (string): Connection URI (e.g., `nhtml_sessions.db`).

---

## 🛠️ `[dev]`
Development-specific features.
*   **`auto_reload`** (bool): Enables/Disables watcher-based hot reload.

---

## Example `nhtml.config.toml`
```toml
[ports]
ws = 8080
php = 8000
devtools = 8081

[fastcgi]
address = "127.0.0.1:9000"
timeout_ms = 3000

[security.tls]
enabled = false
cert = "cert.pem"
key = "key.pem"

[security.rate_limit]
events_per_sec = 20

[database]
driver = "sqlite"
uri = "nhtml_sessions.db"

[dev]
auto_reload = true
```
