# PHP Installation Guide (Server-Side)

This guide explains how to integrate Nhtml templates into a PHP application.

## Prerequisites
- PHP 7.4 or higher.
- `ffi` extension enabled (optional but recommended for speed).
- `exec()` function authorized (for fallback mode).

## Step 1: Download Binaries
Download the latest release and copy these files to your project:
- `nhtml_engine/NhtmlCompiler.php` (The adapter)
- `nhtml.dll` (Windows) or `libnhtml_core.so` (Linux) (The core)

## Step 2: Minimal Implementation
```php
<?php
require_once 'nhtml_engine/NhtmlCompiler.php';
use Nhtml\NhtmlCompiler;

// 1. Read source
$source = file_get_contents('views/home.nhtml');

// 2. Compile (Automatic FFI or CLI fallback)
$result = NhtmlCompiler::compile($source);

// 3. Render
echo $result['html'];
echo "<script>window._nhtmlAST = " . json_encode($result['manifest']) . ";</script>";
```

## Troubleshooting
- **Error: FFI not found**: Install the FFI extension or ensure the binary is in the correct path for CLI fallback.
- **Permission Denied**: Ensure the web server has execution rights on the `nhtml` binary.

---

# 🇫🇷 Guide d'Installation PHP (Côté Serveur)

Ce guide explique comment intégrer les templates Nhtml dans une application PHP.

## Prérequis
- PHP 7.4 ou supérieur.
- Extension `ffi` activée (optionnel mais recommandé pour la vitesse).
- Fonction `exec()` autorisée (pour le mode de secours).

## Étape 1 : Téléchargement des Binaires
Téléchargez la dernière version et copiez ces fichiers dans votre projet :
- `nhtml_engine/NhtmlCompiler.php` (L'adaptateur)
- `nhtml.dll` (Windows) ou `libnhtml_core.so` (Linux) (Le cœur du moteur)

## Étape 2 : Implémentation Minimale
```php
<?php
require_once 'nhtml_engine/NhtmlCompiler.php';
use Nhtml\NhtmlCompiler;

// 1. Lire la source
$source = file_get_contents('views/home.nhtml');

// 2. Compiler (Bascule automatique entre FFI et CLI)
$result = NhtmlCompiler::compile($source);

// 3. Affichage
echo $result['html'];
echo "<script>window._nhtmlAST = " . json_encode($result['manifest']) . ";</script>";
```

## Dépannage
- **Erreur : FFI introuvable** : Installez l'extension FFI ou assurez-vous que le binaire est dans le bon chemin pour la bascule CLI.
- **Permission Refusée** : Assurez-vous que le serveur web a les droits d'exécution sur le binaire `nhtml`.
