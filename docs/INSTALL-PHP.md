# 🐘 NHTML PHP SDK Installation Guide (v0.2.2)

This guide explains how to use the NHTML Professional PHP SDK to drive your user interface through the high-performance NBPS binary protocol.

## Prerequisites
- PHP 8.1 or higher (Recommended for typed properties and performance).
- The NHTML Gateway (Rust) running on your server.

## Step 1: Include the SDK
For now, copy the `sdk/php/src` folder into your project's vendor or library directory.

```php
require_once 'Nhtml/SDK/Protocol/OpCodes.php';
require_once 'Nhtml/SDK/Protocol/Encoder.php';
require_once 'Nhtml/SDK/Gateway.php';
require_once 'Nhtml/SDK/GatewayFactory.php';

use Nhtml\SDK\GatewayFactory;
```

## Step 2: Minimal Implementation
The PHP SDK uses a **Fluent API** to build binary patches sent to the browser via the Gateway.

```php
<?php
// Initialize the Gateway (it handles binary headers automatically)
$nhtml = GatewayFactory::create();

// Build your UI updates
$nhtml->setText(42, "Hello from PHP v0.2.2!")
      ->addClass(10, "is-active")
      ->send(); // Generates and outputs the NBPS binary buffer
```

## Key Concepts
1. **NodeIDs** : Every element in your NHTML template has a unique numerical ID.
2. **Binary Protocol** : Communication is done via `application/octet-stream`. No more JSON parsing overhead!
3. **Stateless Logic** : PHP doesn't need to know the whole DOM, it only sends deltas (patches).

---

# 🇫🇷 Guide d'Installation SDK PHP NHTML (v0.2.2)

Ce guide explique comment utiliser le SDK PHP professionnel pour piloter votre interface via le protocole binaire NBPS.

## Prérequis
- PHP 8.1 ou supérieur.
- Le Gateway NHTML (Rust) actif sur votre serveur.

## Étape 1 : Inclusion du SDK
Copiez le dossier `sdk/php/src` dans votre projet.

```php
require_once 'Nhtml/SDK/GatewayFactory.php'; // Charge le reste automatiquement
use Nhtml\SDK\GatewayFactory;
```

## Étape 2 : Implémentation Minimale
```php
<?php
$nhtml = GatewayFactory::patch();

$nhtml->setText(42, "Salut depuis PHP Binaire !")
      ->addClass(10, "visible")
      ->send();
```

---
**Status :** Industriel. Utilisez ce SDK pour toutes les applications de production NHTML.
