# NHTML Specification v0.4.0 (Binary B-TREE / BIND)

Nhtml (Native HTML) is a reactive template language and a high-performance framework designed for seamless full-stack interactions without writing JavaScript.

## 1. Document Structure
Nhtml files (`.nhtml`) are standard HTML files enriched with `n-*` attributes.
The architecture relies on a **Rust Gateway**, a **PHP Backend**, and a lightweight **JS Bridge**.

## 2. Event Handling (`n-*` attributes)
All interactions are declarative and defined using `n-*` attributes. 
When an event occurs, the JS Bridge sends a binary `EVENT` packet to the Rust Gateway, which dispatches it to the PHP Backend.

```html
<!-- Trigger a PHP function on click -->
<button n-id="btn_submit" n-click="submit_form">Submit</button>

<!-- Send input value on keystroke -->
<input n-id="username" n-input="validate_user" />

<!-- Send event on Enter key -->
<input n-id="chat_input" n-keydown="send_message" />
```

## 3. Local Actions
Local actions provide instant visual feedback without network round-trips to the server. They are parsed by the Rust Compiler and sent to the client via `BIND` packets.

```html
<!-- Toggle a CSS class on click -->
<div n-toggle="active">Click me</div>

<!-- Toggle visibility of another element -->
<button n-toggle-target="menu_id">Toggle Menu</button>

<!-- Apply a CSS class on hover -->
<div n-hover="highlight">Hover me</div>
```

## 4. Architecture & Protocol (NBPS v0.4.0)
Nhtml follows a **Binary Protocol (NBPS - NHTML Binary Protocol System)**:

1. **Rust Gateway**:
   - Parses the `.nhtml` files using the internal **Compiler**.
   - Generates a **B-TREE** (binary representation of the DOM state) and a **Handler Table**.
   - Maintains WebSocket connections with clients and spawns PHP processes to handle logic.

2. **PHP SDK**:
   - Receives state and event data via `stdin` (JSON).
   - Generates DOM mutations using the SDK (`$nhtml->patch()`).
   - Outputs patches via `stdout` (JSON) which the Gateway translates into binary `PATCH` packets.

3. **JS Bridge (`bridge.js`)**:
   - Intercepts DOM events (click, input, keydown).
   - Sends `EVENT` binary packets (0x02).
   - Receives binary `PATCH` packets (0x03) and hydrates the DOM efficiently.
   - Executes Local Actions directly.

## 5. Binary Packet Types
| OpCode | Name   | Description |
|:-------|:-------|:------------|
| `0x01` | HELLO  | Initial handshake and session establishment. |
| `0x02` | EVENT  | Sent by client on user interaction (click, input). |
| `0x03` | PATCH  | Sent by server to mutate the DOM (setText, addClass). |
| `0x04` | BIND   | Sent by server on init to attach event listeners and local actions. |
| `0x05` | SYNC   | Sent periodically by server to verify DOM checksum. |
| `0x10` | LOG    | Sent by server for DevTools mirroring. |

---

# 🇫🇷 Spécification NHTML v0.4.0 (B-TREE Binaire / BIND)

Nhtml (Native HTML) est un langage de template réactif et un framework haute performance conçu pour des interactions full-stack fluides sans écrire de JavaScript.

## 1. Structure du Document
Les fichiers Nhtml (`.nhtml`) sont des fichiers HTML standards enrichis avec des attributs `n-*`.
L'architecture repose sur une **Gateway Rust**, un **Backend PHP** et un **Bridge JS** très léger.

## 2. Gestion des Événements (attributs `n-*`)
Toutes les interactions sont déclaratives.
Lorsqu'un événement survient, le Bridge JS envoie un paquet binaire `EVENT` à la Gateway Rust, qui le transmet au Backend PHP.

```html
<!-- Déclenche une fonction PHP au clic -->
<button n-id="btn_submit" n-click="submit_form">Envoyer</button>

<!-- Envoie la valeur saisie lors de la frappe -->
<input n-id="username" n-input="validate_user" />

<!-- Envoie un événement sur la touche Entrée -->
<input n-id="chat_input" n-keydown="send_message" />
```

## 3. Actions Locales (Local Actions)
Les actions locales offrent un retour visuel instantané sans faire d'aller-retour réseau vers le serveur. Elles sont parsées par le Compilateur Rust et envoyées au client via des paquets `BIND`.

```html
<!-- Bascule une classe CSS au clic -->
<div n-toggle="active">Cliquez-moi</div>

<!-- Bascule la visibilité d'un autre élément -->
<button n-toggle-target="menu_id">Ouvrir le Menu</button>

<!-- Applique une classe CSS au survol -->
<div n-hover="highlight">Survolez-moi</div>
```

## 4. Architecture & Protocole (NBPS v0.4.0)
Nhtml suit un **Protocole Binaire (NBPS)** :

1. **Gateway Rust** :
   - Parse les fichiers `.nhtml` avec le **Compilateur** interne.
   - Génère un **B-TREE** (représentation binaire de l'état du DOM) et une **Table de Handlers**.
   - Maintient les WebSockets et lance les processus PHP.

2. **SDK PHP** :
   - Reçoit l'état et l'événement via `stdin` (JSON).
   - Génère des mutations DOM via le SDK (`$nhtml->patch()`).
   - Renvoie les patchs via `stdout` (JSON) traduits en paquets binaires `PATCH` par la Gateway.

3. **Bridge JS (`bridge.js`)** :
   - Intercepte les événements DOM (click, input, keydown).
   - Envoie des paquets binaires `EVENT` (0x02).
   - Reçoit les paquets binaires `PATCH` (0x03) et met à jour le DOM.
   - Exécute les Actions Locales directement.

---
© 2026 NemStudio
