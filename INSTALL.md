# Tutoriel d'Installation & Déploiement : Nhtml V2

Ce guide vous explique comment intégrer les fichiers `.nhtml` dans vos projets. Le moteur Nhtml V2 se décompose en **3 modes d'exécution** qui s'adaptent à l'environnement hébergeant votre site. 
*(Les 3 modes ne sont pas censés être utilisés en même temps dans la même application. Vous choisissez le plus adapté !)*

---

## 🏗️ 1. Mode "Serveur Dédié" (PHP FFI)
**Idéal pour :** Les VPS, Serveurs Dédiés, et Infrastructures Cloud maîtrisées.
C'est le mode le plus rapide, qui charge Rust directement dans la mémoire de PHP.

### Prérequis
- PHP 7.4 ou supérieur.
- L'extension `ffi` activée dans votre `php.ini` (`ffi.enable=true`).
- Le fichier dynamique compilé `nhtml_core.dll` (Windows) ou `libnhtml_core.so` (Linux).

### Utilisation avec un fichier `.nhtml`
```php
<?php
require_once 'nhtml_engine/NhtmlCompiler.php';
use Nhtml\NhtmlCompiler;

// 1. Lire votre fichier composant
$source = file_get_contents('components/button.nhtml');

// 2. Le compilateur (si FFI est actif) le compile sans temps de latence
$result = NhtmlCompiler::compile($source);

// 3. Injecter l'HTML et le Manifeste dans la vue
echo $result['html'];
echo "<script> window._nhtmlAST = " . json_encode($result['manifest']) . "; </script>";
```

---

## 🐢 2. Mode "Hébergement Mutualisé" (PHP Exec Fallback)
**Idéal pour :** OVH, O2Switch, ou tout vieil hébergement où les extensions C ne peuvent pas être activées.
Si l'extension FFI n'est pas détectée, `NhtmlCompiler.php` basculera automatiquement sur ce mode.

### Prérequis
- La fonction `exec()` autorisée en PHP.
- Le binaire exécutable `nhtml.exe` (Windows) ou `nhtml` (Linux).

### Utilisation avec un fichier `.nhtml`
Exactement la même chose qu'au dessus ! La magie de `NhtmlCompiler.php` est qu'il détecte automatiquement votre environnement. Votre code PHP reste le même.

---

## 🛸 3. Mode "PWA / Front-End Pur" (WebAssembly JS)
**Idéal pour :** Les Single Page Applications (SPA), ou les intégrations dans Next.js, Nuxt, VanillaJS sans passer par un serveur PHP. Le rendu est 100% exécuté par le navigateur de l'utilisateur.

### Prérequis
- Le dossier empaqueté `pkg/` généré par `wasm-pack`.
- Un serveur web standard capable de fournir des fichiers `.wasm` (via Mime-Type).

### Utilisation avec un fichier `.nhtml`
Plutôt que d'attendre l'HTML depuis un serveur, c'est le navigateur qui charge le ".nhtml" !

```javascript
/* Typique d'une application côté client */
import init, { compile_wasm } from './nhtml-core/pkg/nhtml_core.js';

async function initAndRender() {
    // 1. Initialiser le Wasm
    await init();
    
    // 2. Fetcher le fichier purement statique
    const response = await fetch('./views/profil.nhtml');
    const source = await response.text();
    
    // 3. Demander au moteur Wasm de calculer la vue et la logique
    const result_json = compile_wasm(source);
    const result = JSON.parse(result_json);
    
    // 4. Injecter les données et relier le JS de l'utilisateur
    document.getElementById('app').innerHTML = result.html;
    hydrateDOM(result.html, result.manifest); 
}

initAndRender();
```

---

## 💡 En Résumé : Comment ça marche ensemble ?
Ils ne "marchent" pas ensemble : ils proposent **la même interface universelle** (`Compiler(Code) -> HTML + JSON`), afin que vous puissiez écrire vos composants ou vues **une seule fois** (dans `profil.nhtml` par exemple). 
Si un jour vous abandonnez PHP pour Node.js, ou l'inverse, vos fichiers UI restent valables, et seul l'Adaptateur Nhtml change !
