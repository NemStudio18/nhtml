# 🛰️ NHTML — The Server-Driven DOM Protocol

<p align="center">
  <img src="assets/logo.png" width="250" alt="NHTML Logo">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Gateway-Rust-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/Backend-PHP_8.x-777bb4?style=flat-square&logo=php" />
  <img src="https://img.shields.io/badge/Transport-Binary_NBPS-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/JS_métier-Zéro-success?style=flat-square" />
  <img src="https://img.shields.io/badge/License-MIT_/_AGPL-lightgrey?style=flat-square" />
</p>

---

## ⚡ Le web réactif. Sans écrire de JavaScript.

**NHTML est un framework Server-Driven UI.** Vous écrivez du HTML augmenté et du PHP. Le Gateway Rust s'occupe du reste — transport binaire, mutations DOM atomiques, temps réel natif.

Zéro React. Zéro Vue. Zéro build step. Zéro JS métier à écrire.

---

## 🧠 À quoi ça ressemble

**Côté interface (`index.nhtml`)** — du HTML avec des super-pouvoirs :

```html
<h1 n-id="titre">Bonjour</h1>

<p>
  Tu as cliqué
  <strong n-id="compteur">0</strong> fois.
</p>

<button n-click="page.incrementer">Cliquer ici</button>

<div n-id="message" n-live></div>
```

**Côté serveur (`app.php`)** — du PHP pur :

```php
function page_incrementer(NhtmlEvent $event): array
{
    $count = ++$_SESSION['clicks'];

    return [
        Patch::setText('compteur', (string)$count),
        Patch::setText('message', $count === 10 ? '🎉 Dixième clic !' : ''),
    ];
}
```

👉 Pas de JSON à écrire. Pas de fetch(). Pas d'état côté client à gérer.

---

## 🔥 Pourquoi NHTML existe

Le développement web moderne est devenu inutilement complexe :

| Problème | Solution NHTML |
|----------|---------------|
| React/Vue → lourds, complexes, JS partout | HTML augmenté (`n-click`, `n-id`) |
| Build tools → fragiles, lents | Zéro build step |
| State management → dette technique | État géré par PHP côté serveur |
| Debugging frontend → aveugle | DevTools avec Time Travel intégré |
| Déploiement → infra complexe | Un binaire Rust, un script PHP |

---

## ⚔️ NHTML vs les alternatives

| | React | HTMX | **NHTML** |
|---|---|---|---|
| JS métier à écrire | ✅ Beaucoup | ⚠️ Un peu | ❌ Zéro |
| Transport | JSON/REST | HTML texte | **Binaire NBPS** |
| Mutations DOM | Virtual DOM | innerHTML swap | **Atomiques (SET_TEXT, ADD_CLASS…)** |
| DevTools | React DevTools | ❌ Aucun | **Time Travel + Network Monitor** |
| Déploiement mutualisé | ❌ Non | ✅ Oui | ✅ **Oui (fallback HTTP)** |
| Temps réel natif | Via libs | SSE/WS manuel | **WebSocket natif** |

---

## 🏛️ Comment ça marche

```
Votre .nhtml + app.php
        │
        ▼
[ Gateway Rust ]  ←──────────────────────────────┐
        │                                         │
        │  WebSocket (Protocole NBPS Binaire)     │
        │                                         │
        ▼                                         │
[ Navigateur ]                              [ PHP Backend ]
  bridge.js (~2KB)    EVENT (clic) ────────▶  Logique métier
  Applique les PATCH  PATCH (mutations) ◀────  Renvoie les ops
```

1. **Vous écrivez** un fichier `.nhtml` (HTML + attributs `n-`) et un `app.php`
2. **Le Gateway** compile le `.nhtml` en arbre binaire (B-TREE), détecte les `n-click`, `n-id`
3. **Le navigateur** reçoit le binaire, construit le DOM, attache les listeners
4. **Au clic** : un paquet binaire de quelques octets part au Gateway
5. **PHP traite** et retourne une liste de mutations (`Patch::setText`, `Patch::addClass`…)
6. **Le DOM est mis à jour** chirurgicalement — pas de rechargement, pas de re-render

---

## 🎯 Attributs `n-` disponibles

| Attribut | Rôle |
|----------|------|
| `n-id="nom"` | Identifiant métier — cible des mutations PHP |
| `n-click="handler"` | Envoie un EVENT au clic |
| `n-submit="handler"` | Envoie un EVENT à la soumission |
| `n-input="handler"` | Envoie un EVENT à chaque frappe |
| `n-model="var"` | Binding bidirectionnel input ↔ PHP |
| `n-live` | Zone mise à jour par push serveur |
| `n-debounce="300"` | Délai avant envoi (ms) |
| `n-prevent` | preventDefault() automatique |

---

## 🚀 Démarrage rapide

### 1. Télécharger le Gateway

```bash
# Linux / Mac
curl -L https://github.com/NemStudio18/nhtml-gateway/releases/latest/download/nhtml-linux -o nhtml
chmod +x nhtml

# Windows
# Télécharger nhtml.exe depuis les Releases
```

### 2. Créer votre premier projet

```bash
mkdir mon-app && cd mon-app
```

**`index.nhtml`** :
```html
<!DOCTYPE html>
<html>
<body>
  <h1 n-id="titre">Hello NHTML</h1>
  <button n-click="page.saluer">Cliquer</button>
</body>
</html>
```

**`app.php`** :
```php
<?php
require_once 'vendor/autoload.php';
use Nhtml\Patch;

function page_saluer($event): array {
    return [
        Patch::setText('titre', 'Bonjour depuis PHP ! 👋'),
    ];
}
```

### 3. Lancer

```bash
./nhtml start --dev
# → App sur http://127.0.0.1:3000
# → DevTools sur http://127.0.0.1:8081
```

---

## 🛠️ DevTools intégrés

NHTML inclut une station de contrôle complète accessible sur `http://127.0.0.1:8081` :

- **Network Monitor** — chaque paquet NBPS en temps réel
- **Time Travel** — rejouer n'importe quelle session action par action
- **Node Inspector** — état binaire de chaque nœud DOM
- **State Diff Viewer** — visualiser les mutations avant/après
- **Handler Tracer** — profiler la latence de vos handlers PHP
- **Payload Tester** — injecter des EVENTs manuellement (style Postman)

---

## 🏗️ Les 3 modes de déploiement

| Mode | Transport | Prérequis | Cas d'usage |
|------|-----------|-----------|-------------|
| **Dédié** | WebSocket | VPS + binaire Gateway | Temps réel, performance max |
| **Mutualisé** | HTTP POST | Hébergement standard (OVH…) | Apps classiques sans WebSocket |
| **WASM (Zéro-Serveur)** | Local | GitHub Pages / Statique | 🟢 **Opérationnel (Zéro CDN)** |

---

## 📦 Structure d'un projet NHTML

```
mon-app/
├── index.nhtml        ← Interface (HTML + attributs n-)
├── app.php            ← Logique métier PHP
├── nhtml.config.toml  ← Configuration des ports
├── nhtml              ← Binaire Gateway (ou nhtml.exe)
└── vendor/            ← SDK PHP (Composer)
```

---

## 📚 Documentation

| Document | Contenu |
|----------|---------|
| [`docs/SPEC.md`](./docs/SPEC.md) | Protocole NBPS — référence complète |
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) | Architecture et flux fonctionnel |
| [`docs/GUIDE_DEMARRAGE.md`](./docs/GUIDE_DEMARRAGE.md) | Guide d'installation détaillé |
| [`docs/DEPLOIEMENT.md`](./docs/DEPLOIEMENT.md) | Nginx / Apache / Systemd |
| [`docs/INTERNALS.md`](./docs/INTERNALS.md) | Contributeurs Rust |
| [`CHANGELOG.md`](./CHANGELOG.md) | Historique des versions |
| [`CONTRIBUTING.md`](./CONTRIBUTING.md) | Comment contribuer |

---

## 🌍 Cas d'usage idéaux

- ✔ Tableaux de bord temps réel
- ✔ Interfaces CRUD / Admin panels
- ✔ CMS (voir [NCMS](https://github.com/NemStudio18/NCMS))
- ✔ Formulaires avec validation live
- ✔ Applications où React est **trop lourd**

---

## ⚖️ Licence

- **NHTML Core** (SDK PHP, bridge.js) — [MIT](./LICENSE)
- **NHTML Gateway** (serveur Rust) — AGPL v3

> Pour des déploiements cloud propriétaires, contactez NemStudio.

---

<p align="center">
  © 2026 NemStudio18 — Built for simplicity. Powered by Rust.
</p>
