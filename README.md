# 🛰️ NHTML — Server-Driven UI Runtime

<p align="center">
  <img src="assets/logo.png" width="220" alt="NHTML Logo">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Version-v0.7.3--stable-ff007f?style=flat-square" />
  <img src="https://img.shields.io/badge/Gateway-Rust%20%2B%20Tokio-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/Protocol-NBPS%20Binary-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/Backend-PHP%20%7C%20Python%20%7C%20Go%20%7C%20Node-777?style=flat-square" />
  <img src="https://img.shields.io/badge/License-AGPLv3%20%2F%20MIT%20%2F%20Commercial-red?style=flat-square" />
</p>

<p align="center">
  <a href="#-english-version">🇬🇧 English</a> · <a href="#-version-française">🇫🇷 Français</a>
</p>

---

## 🇬🇧 English Version

### What is NHTML?

**NHTML is a Server-Driven UI runtime.** It lets your backend (PHP, Python, Go, Node) drive the browser's DOM directly — in real time — through a binary WebSocket protocol called **NBPS**, without the developer writing a single line of business JavaScript.

It is made of three indivisible layers:

| Layer | What it is | License |
|-------|-----------|---------|
| **NBPS Protocol** | A binary packet specification for atomic DOM mutations over WebSocket | Open spec |
| **Gateway (Rust)** | The high-performance runtime: WebSocket server, PHP supervisor, session manager, DevTools | AGPL v3 |
| **SDKs** | Adapters for PHP, Python, Go, Node.js — build patches in your language | MIT |

> **Not a framework. Not just a server. A protocol and its runtime.**
> NHTML occupies the same space as Phoenix LiveView (Elixir) or Laravel Livewire (PHP),
> but with a Rust binary transport layer that operates independently of your backend language.

---

### How it compares

| | NHTML | Livewire | LiveView | HTMX |
|--|-------|----------|----------|------|
| Transport | Binary (NBPS) | HTTP/WebSocket | WebSocket | HTTP |
| DOM update | Atomic binary ops | HTML diff | HTML diff | HTML swap |
| Backend | PHP, Python, Go, Node | PHP only | Elixir only | Any |
| JS to write | Zero | Zero | Zero | Zero |
| Zero-Server mode | ✅ (PHP-WASM) | ❌ | ❌ | ❌ |
| Built-in DevTools | ✅ | Partial | Partial | ❌ |

---

### The problem NHTML solves

Every modern reactive web stack forces you to choose:

- **Write JavaScript** (React, Vue, Svelte) → complex frontend, duplicated business logic
- **Use HTTP morphing** (HTMX, Turbo) → page-level granularity, no fine-grained real-time
- **Lock into one backend** (Livewire → PHP, LiveView → Elixir) → no flexibility

**NHTML decouples the transport from the backend.** Your PHP, Python or Go code returns a list of DOM operations. The Gateway applies them in the browser at binary speed.

---

### What it looks like

**Template (`index.nhtml`)** — HTML with behaviour attributes:

```html
<h1 n-id="title">Hello</h1>

<p>You clicked <strong n-id="counter">0</strong> times.</p>

<button n-click="page.increment">Click</button>

<div n-id="message" n-live></div>
```

**Backend (`app.php`)** — Pure PHP, no framework required:

```php
<?php
require_once 'sdk/php/src/Nhtml.php';
require_once 'sdk/php/src/Patch.php';
use Nhtml\Nhtml;

$input   = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$nodes   = $input['nodes']   ?? [];

$count = (int)($nodes['counter']['val'] ?? 0);

if ($handler === 'page.increment') {
    $count++;
}

Nhtml::patch()
    ->setText('counter', (string)$count)
    ->setText('message', $count === 10 ? '🎉 Tenth click!' : '')
    ->send();
```

**What happens under the hood:**

```
index.nhtml + app.php
       │
       ▼
[ NHTML Gateway (Rust) ]
       │  Compiles .nhtml → Binary B-TREE
       │  Maintains DOM state in SQLite
       │  HMAC-signs every packet
       │
       │  WebSocket — NBPS Binary Protocol (Zstd compressed)
       │
  ┌────┴────┐
  │ Browser │  ← bridge.js (~25KB, no dependencies)
  │         │    Applies atomic PATCH opcodes to the DOM
  └────┬────┘
       │  EVENT (click, input, submit...)
       ▼
[ PHP / Python / Go / Node ]
    Your business logic
    Returns PATCH operations
       │
       ▼
[ Gateway broadcasts to all relevant sessions ]
```

---

### The NBPS Protocol — at a glance

NBPS (Native Binary Packet Specification) is the wire format NHTML uses. It is **not HTTP, not JSON, not a WebSocket subprotocol** — it is a custom binary format:

```
Packet: [Type:1 byte][Length:4 bytes][Payload:N bytes]

Packet types:
  0x01  HELLO   — Session init, HMAC secret exchange
  0x02  EVENT   — User interaction (click, input...) + HMAC signature
  0x03  PATCH   — Batch of DOM mutations
  0x04  BIND    — Node registration + local actions
  0x05  SYNC    — DOM checksum verification
  0x07  BTREE   — Full DOM snapshot (Zstd compressed + CRC32)
  0x08  PUSH_PATCH — Client-to-server patch (Zero-Server mode)
  0x09  PING    — Keepalive
  0x10  LOG     — Server-to-client log message
  0x7F  ERR     — Error packet

DOM operations (inside PATCH):
  0x01 SET_TEXT      0x02 SET_ATTR     0x03 DEL_ATTR
  0x04 ADD_CLASS     0x05 DEL_CLASS    0x06 INSERT_BEFORE
  0x07 INSERT_AFTER  0x08 REMOVE       0x09 SET_STYLE
  0x0A REPLACE_INNER 0x0B APPEND_HTML  0x0C SCROLL_TO
  0x0D FOCUS
```

Every EVENT packet carries a **HMAC-SHA256 signature** and a **sequence ID** preventing replay attacks.
Every B-TREE snapshot is **Zstd-compressed** and **CRC32-verified**.

---

### Three deployment modes

NHTML adapts to your infrastructure **without changing a line of your code**:

#### 1. 🚀 Dedicated — Rust Gateway (recommended)

Best performance. The binary handles WebSockets, HTTP, PHP supervision, DevTools, and Redis clustering.

```bash
./nhtml start --dev                        # Development
./nhtml start --fpm 127.0.0.1:9000        # Production with PHP-FPM
./nhtml start --fpm unix:/run/php-fpm.sock # Production via Unix Socket
```

**Gateway features:**
- FastCGI pool with load balancing (round-robin or least-connections)
- Auto-restart supervisor for PHP processes (exponential backoff)
- LRU-bounded rate limiter (2048 IPs, O(1) memory)
- Path traversal protection via `canonicalize()` + prefix check
- Redis cluster sync for horizontal scaling
- Built-in DevTools on `http://127.0.0.1:8082` (dev mode only)

#### 2. 🌐 Shared hosting — Standard PHP

No binary needed. Works on any PHP hosting (OVH, cPanel, shared servers).
Uses HTTP polling or SSE fallback. Configure your `router.php` and serve.

#### 3. 🧪 Zero-Server — PHP-WASM

No server at all. The PHP runtime compiles to WebAssembly and runs in the browser.
Ideal for static hosting (GitHub Pages, Netlify, Cloudflare Pages).

```html
<!-- bridge.js detects the mode automatically -->
<script src="assets/js/bridge.js" data-mode="wasm"></script>
```

---

### Quick Start

```bash
# 1. Download the binary for your OS from Releases
# 2. Create a new project
./nhtml new my-project
cd my-project

# 3. Start in development mode
./nhtml start --dev

# 4. Open http://localhost:8080
```

**Project structure:**

```
my-project/
├── index.nhtml          ← Your template (HTML + n-* attributes)
├── app.php              ← Your backend logic
├── nhtml.config.toml    ← Gateway configuration
└── assets/
    └── js/
        └── bridge.js    ← Client runtime (auto-served by Gateway)
```

**Minimal `nhtml.config.toml`:**

```toml
[ports]
ws       = 8080   # WebSocket + HTTP
php      = 8000   # PHP dev server
devtools = 8082   # DevTools dashboard (dev only)

[security]
# Restrict WebSocket origin (CSWH protection) — required in production
allowed_origins = ["https://your-domain.com"]

# Rate limiting (events per second per IP)
[security.rate_limit]
events_per_sec = 30

[fastcgi]
# PHP-FPM address for production (comment out for dev CGI mode)
# address = "127.0.0.1:9000"
timeout_ms = 5000
```

---

### DevTools

NHTML ships with a complete diagnostic interface at `http://127.0.0.1:8082` (activated with `--dev`):

![DevTools Preview](assets/devtools_preview.png)

| Feature | Description |
|---------|-------------|
| **Network Monitor** | Every NBPS packet in real time (type, size, latency, session) |
| **Time Travel** | Replay any session action by action |
| **Node Inspector** | Binary state of every DOM node |
| **State Diff Viewer** | Before/after visualization of every mutation |

> DevTools are **disabled in production** by default. They only start when `--dev` is passed
> and are bound to `127.0.0.1` only. A unique token is auto-generated at each start.

---

### SDK Reference

NHTML provides first-class SDKs for four backend languages. All SDKs produce the same binary PATCH operations.

**PHP** (most complete):
```php
use Nhtml\Nhtml;
Nhtml::patch()
    ->setText('counter', '42')
    ->addClass('button', 'active')
    ->setAttr('input', 'disabled', 'true')
    ->send();

// Broadcast to all sessions in a room
Nhtml::patch()->setText('status', 'online')->broadcastToRoom('lobby')->send();
```

**Python (FastAPI)**:
```python
from nhtml import Nhtml
patch = Nhtml.patch()
patch.set_text('counter', '42')
patch.add_class('button', 'active')
patch.send()
```

**Node.js**:
```javascript
const { Nhtml } = require('./sdk/nodejs');
Nhtml.patch().setText('counter', '42').send();
```

**Go**:
```go
import "nhtml/sdk/go"
nhtml.Patch().SetText("counter", "42").Send()
```

---

### CLI Reference

```bash
nhtml new <name>          Create a new project with correct structure and template
nhtml start               Start the Gateway
  --dev                   Enable watcher + DevTools + live reload
  --port <n>              WebSocket port (overrides config, default: 8080)
  --fpm <addr>            PHP-FPM address (production mode)
  --path <dir>            Project directory (default: .)
  --json                  Structured JSON logs (for ELK / Datadog)
nhtml build               Compile project for production
  --production            Maximum optimization
  --output <dir>          Output directory (default: dist)
nhtml share               Expose local project via secure tunnel (requires confirmation)
nhtml inspect <hex>       Decode a raw NBPS packet (hex string)
nhtml validate <file>     Validate a binary NBPS file
nhtml bench <file>        Compare binary vs HTML size metrics
nhtml devtools            Start DevTools dashboard standalone
```

---

### Architecture decisions

**Why Rust for the Gateway?**
The Gateway is a network multiplexer, not an application server. It maintains thousands of WebSocket connections concurrently, each with independent state. Rust's async runtime (Tokio) and zero-cost abstractions make it the right tool — no GC pauses, predictable latency, single static binary.

**Why a binary protocol instead of JSON?**
JSON over WebSocket is readable but wasteful. A `SET_TEXT` operation in JSON is ~50 bytes minimum. In NBPS it is 7 bytes (2 target ID + 1 opcode + 4 version) plus the string length. For a real-time UI with 60 events/second, this compounds quickly. The binary format also enables native Zstd compression of B-TREE snapshots — typically 70% size reduction.

**Why keep PHP as the backend?**
PHP runs everywhere, has mature tooling, and developers already know it. The Gateway is backend-agnostic by design — the same binary runs with PHP, Python, Go or Node. Locking the Gateway to one language would defeat the purpose.

---

### License

NHTML uses a **Triple-License** model:

| Component | License | Use case |
|-----------|---------|----------|
| SDKs (PHP, Python, Go, Node) + `bridge.js` | **MIT** | Any project, commercial or open-source |
| Gateway Core (Rust) | **AGPL v3** | Open-source projects; contributions flow back |
| Gateway Core (Rust) | **Commercial** | Proprietary / closed-source deployments |

See `LICENSE_MIT`, `LICENSE_AGPL.txt`, and `LICENSE_COMMERCIAL.txt` for full terms.

---

## 🇫🇷 Version Française

### Qu'est-ce que NHTML ?

**NHTML est un runtime Server-Driven UI.** Il permet à votre backend (PHP, Python, Go, Node) de piloter directement le DOM du navigateur — en temps réel — via un protocole WebSocket binaire appelé **NBPS**, sans que le développeur écrive une seule ligne de JavaScript métier.

Il est constitué de trois couches indissociables :

| Couche | Ce que c'est | Licence |
|--------|-------------|---------|
| **Protocole NBPS** | Une spécification de paquets binaires pour les mutations DOM atomiques sur WebSocket | Spec ouverte |
| **Gateway (Rust)** | Le runtime haute performance : serveur WebSocket, superviseur PHP, gestionnaire de sessions, DevTools | AGPL v3 |
| **SDKs** | Adaptateurs pour PHP, Python, Go, Node.js — construisez des patches dans votre langage | MIT |

> **Ce n'est pas un framework. Ce n'est pas juste un serveur. C'est un protocole et son runtime.**
> NHTML occupe le même espace que Phoenix LiveView (Elixir) ou Laravel Livewire (PHP),
> mais avec une couche de transport binaire Rust qui fonctionne indépendamment de votre langage backend.

---

### Comparaison

| | NHTML | Livewire | LiveView | HTMX |
|--|-------|----------|----------|------|
| Transport | Binaire (NBPS) | HTTP/WebSocket | WebSocket | HTTP |
| Mise à jour DOM | Ops binaires atomiques | Diff HTML | Diff HTML | Swap HTML |
| Backend | PHP, Python, Go, Node | PHP uniquement | Elixir uniquement | Quelconque |
| JS à écrire | Zéro | Zéro | Zéro | Zéro |
| Mode Zéro-Serveur | ✅ (PHP-WASM) | ❌ | ❌ | ❌ |
| DevTools intégrés | ✅ | Partiel | Partiel | ❌ |

---

### Le problème que NHTML résout

Chaque stack web réactif moderne vous force à choisir :

- **Écrire du JavaScript** (React, Vue, Svelte) → frontend complexe, logique métier dupliquée
- **Utiliser du morphing HTTP** (HTMX, Turbo) → granularité page entière, pas de temps réel fin
- **Se verrouiller sur un backend** (Livewire → PHP, LiveView → Elixir) → pas de flexibilité

**NHTML découple le transport du backend.** Votre code PHP, Python ou Go retourne une liste d'opérations DOM. Le Gateway les applique dans le navigateur à vitesse binaire.

---

### À quoi ça ressemble

**Template (`index.nhtml`)** — HTML avec attributs de comportement :

```html
<h1 n-id="titre">Bonjour</h1>

<p>Tu as cliqué <strong n-id="compteur">0</strong> fois.</p>

<button n-click="page.incrementer">Cliquer</button>

<div n-id="message" n-live></div>
```

**Backend (`app.php`)** — PHP pur, aucun framework requis :

```php
<?php
require_once 'sdk/php/src/Nhtml.php';
require_once 'sdk/php/src/Patch.php';
use Nhtml\Nhtml;

$input   = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$nodes   = $input['nodes']   ?? [];

$count = (int)($nodes['compteur']['val'] ?? 0);

if ($handler === 'page.incrementer') {
    $count++;
}

Nhtml::patch()
    ->setText('compteur', (string)$count)
    ->setText('message', $count === 10 ? '🎉 Dixième clic !' : '')
    ->send();
```

**Ce qui se passe sous le capot :**

```
index.nhtml + app.php
       │
       ▼
[ Gateway NHTML (Rust) ]
       │  Compile .nhtml → B-TREE binaire
       │  Maintient l'état DOM en SQLite
       │  Signe chaque paquet HMAC-SHA256
       │
       │  WebSocket — Protocole NBPS Binaire (compressé Zstd)
       │
  ┌────┴──────┐
  │ Navigateur │  ← bridge.js (~25Ko, zéro dépendance)
  │            │    Applique les opcodes PATCH atomiques sur le DOM
  └────┬───────┘
       │  EVENT (clic, input, submit...)
       ▼
[ PHP / Python / Go / Node ]
    Votre logique métier
    Retourne des opérations PATCH
       │
       ▼
[ Gateway diffuse aux sessions concernées ]
```

---

### Le protocole NBPS — en bref

NBPS (Native Binary Packet Specification) est le format réseau utilisé par NHTML. **Ce n'est pas du HTTP, pas du JSON, pas un sous-protocole WebSocket** — c'est un format binaire custom :

```
Paquet : [Type:1 octet][Longueur:4 octets][Payload:N octets]

Types de paquets :
  0x01  HELLO       — Init de session, échange du secret HMAC
  0x02  EVENT       — Interaction utilisateur + signature HMAC
  0x03  PATCH       — Lot de mutations DOM
  0x04  BIND        — Enregistrement de nœud + actions locales
  0x05  SYNC        — Vérification du checksum DOM
  0x07  BTREE       — Snapshot DOM complet (Zstd + CRC32)
  0x08  PUSH_PATCH  — Patch client→serveur (mode Zéro-Serveur)
  0x09  PING        — Keepalive
  0x10  LOG         — Message de log serveur→client
  0x7F  ERR         — Paquet d'erreur

Opérations DOM (dans PATCH) :
  0x01 SET_TEXT      0x02 SET_ATTR     0x03 DEL_ATTR
  0x04 ADD_CLASS     0x05 DEL_CLASS    0x06 INSERT_BEFORE
  0x07 INSERT_AFTER  0x08 REMOVE       0x09 SET_STYLE
  0x0A REPLACE_INNER 0x0B APPEND_HTML  0x0C SCROLL_TO
  0x0D FOCUS
```

Chaque paquet EVENT porte une **signature HMAC-SHA256** et un **sequence ID** empêchant les attaques par rejeu.
Chaque snapshot B-TREE est **compressé Zstd** et **vérifié CRC32**.

---

### Trois modes de déploiement

NHTML s'adapte à votre infrastructure **sans changer une ligne de votre code** :

#### 1. 🚀 Dédié — Gateway Rust (recommandé)

Meilleures performances. Le binaire gère WebSockets, HTTP, supervision PHP, DevTools et clustering Redis.

```bash
./nhtml start --dev                          # Développement
./nhtml start --fpm 127.0.0.1:9000          # Production avec PHP-FPM
./nhtml start --fpm unix:/run/php-fpm.sock  # Production via Unix Socket
```

**Fonctionnalités du Gateway :**
- Pool FastCGI avec load balancing (round-robin ou least-connections)
- Superviseur auto-restart pour les processus PHP (backoff exponentiel)
- Rate limiter LRU borné en mémoire (2048 IPs, O(1))
- Protection anti-path traversal via `canonicalize()` + vérification de préfixe
- Synchronisation Redis cluster pour le scaling horizontal
- DevTools intégrés sur `http://127.0.0.1:8082` (mode dev uniquement)

#### 2. 🌐 Hébergement mutualisé — PHP Standard

Aucun binaire nécessaire. Fonctionne sur tout hébergement PHP (OVH, cPanel, mutualisés).
Utilise le polling HTTP ou SSE en fallback.

#### 3. 🧪 Zéro-Serveur — PHP-WASM

Aucun serveur du tout. Le runtime PHP compilé en WebAssembly tourne dans le navigateur.
Idéal pour les hébergements statiques (GitHub Pages, Netlify, Cloudflare Pages).

```html
<script src="assets/js/bridge.js" data-mode="wasm"></script>
```

---

### Démarrage rapide

```bash
# 1. Télécharger le binaire pour votre OS depuis Releases
# 2. Créer un nouveau projet
./nhtml new mon-projet
cd mon-projet

# 3. Démarrer en mode développement
./nhtml start --dev

# 4. Ouvrir http://localhost:8080
```

**Structure du projet :**

```
mon-projet/
├── index.nhtml          ← Votre template (HTML + attributs n-*)
├── app.php              ← Votre logique backend
├── nhtml.config.toml    ← Configuration du Gateway
└── assets/
    └── js/
        └── bridge.js    ← Runtime client (servi automatiquement)
```

**`nhtml.config.toml` minimal :**

```toml
[ports]
ws       = 8080   # WebSocket + HTTP
php      = 8000   # Serveur PHP dev
devtools = 8082   # Dashboard DevTools (dev uniquement)

[security]
# Restreindre l'origine WebSocket (protection CSWH) — requis en production
allowed_origins = ["https://votre-domaine.com"]

# Rate limiting (événements par seconde par IP)
[security.rate_limit]
events_per_sec = 30

[fastcgi]
# Adresse PHP-FPM pour la production (commenter pour le mode dev CGI)
# address = "127.0.0.1:9000"
timeout_ms = 5000
```

---

### DevTools

NHTML embarque une interface de diagnostic complète sur `http://127.0.0.1:8082` (activée avec `--dev`) :

![DevTools Preview](assets/devtools_preview.png)

| Fonctionnalité | Description |
|----------------|-------------|
| **Network Monitor** | Chaque paquet NBPS en temps réel (type, taille, latence, session) |
| **Time Travel** | Rejouer n'importe quelle session action par action |
| **Node Inspector** | État binaire de chaque nœud DOM |
| **State Diff Viewer** | Visualisation avant/après de chaque mutation |

> Les DevTools sont **désactivés en production** par défaut. Ils ne démarrent qu'avec `--dev`,
> écoutent uniquement sur `127.0.0.1`, et un token unique est auto-généré à chaque démarrage.

---

### Référence SDK

**PHP** (le plus complet) :
```php
use Nhtml\Nhtml;
Nhtml::patch()
    ->setText('compteur', '42')
    ->addClass('bouton', 'actif')
    ->setAttr('input', 'disabled', 'true')
    ->send();

// Diffuser à toutes les sessions d'un salon
Nhtml::patch()->setText('statut', 'en ligne')->broadcastToRoom('lobby')->send();
```

**Python (FastAPI)** :
```python
from nhtml import Nhtml
Nhtml.patch().set_text('compteur', '42').add_class('bouton', 'actif').send()
```

**Node.js** :
```javascript
const { Nhtml } = require('./sdk/nodejs');
Nhtml.patch().setText('compteur', '42').send();
```

**Go** :
```go
import "nhtml/sdk/go"
nhtml.Patch().SetText("compteur", "42").Send()
```

---

### Référence CLI

```bash
nhtml new <nom>           Créer un nouveau projet avec la structure et le template corrects
nhtml start               Démarrer le Gateway
  --dev                   Activer le watcher + DevTools + live reload
  --port <n>              Port WebSocket (priorité sur la config, défaut: 8080)
  --fpm <addr>            Adresse PHP-FPM (mode production)
  --path <rép>            Répertoire du projet (défaut: .)
  --json                  Logs JSON structurés (pour ELK / Datadog)
nhtml build               Compiler le projet pour la production
  --production            Optimisation maximale
  --output <rép>          Répertoire de sortie (défaut: dist)
nhtml share               Exposer le projet local via tunnel sécurisé (demande confirmation)
nhtml inspect <hex>       Décoder un paquet NBPS brut (chaîne hex)
nhtml validate <fichier>  Valider un fichier binaire NBPS
nhtml bench <fichier>     Comparer les métriques binaire vs HTML
nhtml devtools            Démarrer le dashboard DevTools en standalone
```

---

### Décisions d'architecture

**Pourquoi Rust pour le Gateway ?**
Le Gateway est un multiplexeur réseau, pas un serveur applicatif. Il maintient des milliers de connexions WebSocket concurrentes, chacune avec son état propre. Le runtime async de Rust (Tokio) et ses abstractions à coût zéro le rendent idéal — pas de pauses GC, latence prévisible, binaire statique unique.

**Pourquoi un protocole binaire plutôt que JSON ?**
JSON sur WebSocket est lisible mais coûteux. Une opération `SET_TEXT` en JSON fait ~50 octets minimum. En NBPS, c'est 7 octets (2 target ID + 1 opcode + 4 version) plus la longueur de la chaîne. Pour une UI temps réel à 60 événements/seconde, ça s'accumule vite. Le format binaire permet aussi la compression Zstd native des snapshots B-TREE — réduction de taille typique de 70%.

**Pourquoi garder PHP comme backend ?**
PHP tourne partout, dispose d'outillage mature et les développeurs le connaissent. Le Gateway est backend-agnostique par conception — le même binaire fonctionne avec PHP, Python, Go ou Node. Verrouiller le Gateway sur un langage irait à l'encontre de l'objectif.

---

### Licence

NHTML utilise un modèle de **Triple-Licence** :

| Composant | Licence | Cas d'usage |
|-----------|---------|-------------|
| SDKs (PHP, Python, Go, Node) + `bridge.js` | **MIT** | Tout projet, commercial ou open-source |
| Gateway Core (Rust) | **AGPL v3** | Projets open-source ; les contributions reviennent à la communauté |
| Gateway Core (Rust) | **Commerciale** | Déploiements propriétaires / source fermée |

Voir `LICENSE_MIT`, `LICENSE_AGPL.txt`, et `LICENSE_COMMERCIAL.txt` pour les termes complets.

---

<p align="center">
  © 2026 NemStudio18 — Built with Rust. Driven by simplicity.
</p>
