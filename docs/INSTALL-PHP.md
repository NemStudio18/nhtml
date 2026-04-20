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

# 🇫🇷 Guide d'Installation PHP

Ce guide explique comment intégrer les templates Nhtml dans une application PHP.

## 1. Télécharger les fichiers
- `nhtml_engine/NhtmlCompiler.php`
- `nhtml.dll` ou `libnhtml_core.so`

## 2. Exemple de code
```php
require_once 'nhtml_engine/NhtmlCompiler.php';
$result = Nhtml\NhtmlCompiler::compile($source);
echo $result['html'];
```
