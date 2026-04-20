# ⚛️ Nhtml

**Nhtml est HTML avec la réactivité intégrée — sans framework, sans build tool, sans Node.js.**

Nhtml (Native HTML) est un langage de template compilé par un moteur Rust ultra-rapide, conçu pour apporter de l'interactivité moderne à vos applications Web sans la complexité des écosystèmes JavaScript actuels.

```html
<!-- Votre composant .nhtml -->
<var count=0>

<div class="card">
    <h2>Compteur : {count}</h2>
    
    <button on:click="count++">Incrémenter</button>
    <button on:click="count--">Décrémenter</button>
    
    <if condition="count > 10">
        <p>🔥 Score impressionnant !</p>
    </if>
</div>
```

## Pourquoi Nhtml ?

*   🚀 **Performance Native** : Le noyau est écrit en Rust. Compilation instantanée via FFI.
*   🌍 **Zéro Dépendance** : Pas de `npm install`, pas de `node_modules`, pas de Webpack.
*   💎 **Multi-Cibles** : Fonctionne sur Serveur (PHP/C/Rust) et sur Client (WebAssembly).
*   ⛓️ **Réactivité Durable** : Un runtime JS de moins de 2 Ko pour hydrater le DOM.

## Installation Rapide

Consultez le guide complet [INSTALL.md](./INSTALL.md).

### PHP (Serveur)
```php
$result = NhtmlCompiler::compile($source);
echo $result['html'];
```

### WebAssembly (Navigateur)
```javascript
import init, { compile_wasm } from './pkg/nhtml_core.js';
await init();
const res = compile_wasm(source);
```

## Spécifications
Le langage suit la spécification [SPEC.md](./SPEC.md) (v2.0).

---
© 2026 NemStudio — Propulsé par la simplicité.
