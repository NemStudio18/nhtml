# NHTML — Next HyperText Markup Language
## Specification v0.6 — Document de Référence

**Version** : 0.6.0  
**Statut** : Release Candidate (Validation)  
**Licence** : MIT  
**Date** : 17 Avril 2026

---

## 1. Introduction & Vision

Nhtml est un langage déclaratif qui étend le HTML standard pour lui donner des capacités réactives natives sans dépendre de frameworks complexes (React, Vue, etc.). 

**Principes Clés :**
- **Un seul fichier** : Logique, style et structure cohabitent.
- **Transpilation Headless** : Le code Nhtml est traduit en HTML/JS pur compréhensible par n'importe quel navigateur.
- **Réactivité basée sur l'état** : Toute modification de variable met à jour le DOM instantanément.

---

## 2. Nouveautés de la v0.6 (Modularisation & Fiabilité)

La série de versions v0.4, v0.5 et v0.6 marque la transition de Nhtml d'un script Proof-of-Concept vers un véritable moteur de rendu professionnel.

### 2.1 Refonte Architecturale Modulaire (v0.6)
Le transpileur Nhtml n'est plus un monolithe d'un seul fichier (`nhtml.py`) de 1000 lignes. Il a été découpé sous la forme d'un package Python, `nhtml_engine` :
- `cli.py` : Entrées/sorties et gestion en ligne de commande.
- `parser.py` : L'orchestrateur de compilation (`NhtmlParser`).
- `transformer.py` : Les abstractions du langage et les générateurs de code JS/CSS (`NhtmlTransformer`).
- `utils.py` : Utilitaires parseurs HTML natifs.

### 2.2 Scoping JS Intelligent (v0.5)
Dans les boucles `<each>`, la portée des variables est désormais gérée contextuellement. L'itérateur ne remplace et n'écrase la variable parente **que** dans les contextes JavaScript (`${...}`) ou les affectations, préservant ainsi l'intégrité absolue des noms de classes CSS et des URLs non-réactives de la page.

### 2.3 Post-Rendu Réactif (v0.5)
Les éléments créés dynamiquement (typiquement ceux insérés par le biais de balises `<each>`) relaient maintenant un appel `_nhtmlUpdateDOM()` local afin de s'assurer que leurs attributs `data-nhtml-attrs` (comme les `href`) sont injectés directement après leur création dans le cycle du DOM.

### 2.4 Auto-Régénération par Timestamp (v0.5)
Le moteur Nhtml peut désormais être intégré dans un wrapper Backend (comme PHP avec `NhtmlCache.php`) permettant de re-compiler silencieusement vos pages `.nhtml` vers `.html` dès lors qu'il détecte une modification de timestamp. Le besoin d'une exécution manuelle via terminal n'est plus obligatoire.

### 2.5 Objets Javascript "Literals"
L'interpolation `{expression}` détecte les objets JSON structurés et les ignore, de sorte qu'écrire des commentaires JS contenant `{a: 1}` n'interrompt pas la compilation.

---

## 3. Structure d'un fichier Nhtml

Un fichier Nhtml peut contenir les blocs suivants :

| Bloc | Rôle |
| :--- | :--- |
| `<head>` | Métadonnées et liens (standard HTML) |
| `<var>` | Déclaration de variables réactives |
| `<body>` | Structure de la page |
| `<if>` / `<elseif>` / `<else>` | Logique conditionnelle |
| `<each>` | Boucles de rendu |
| `<style>` | Styles CSS (locaux ou globaux) |
| `<script>` | Logique JavaScript |
| `<component>` | Définition de composants réutilisables |

---

## 4. Syntaxe de Réactivité

### 4.1 Variables globales
```html
<var site_name="Mon Blog">
<var counter=0>
```

### 4.2 Interpolation
```html
<h1>{site_name}</h1>
<button on:click="counter++">Clics : {counter}</button>
```

### 4.3 Liaison d'attributs
```html
<div class="status {is_active ? 'on' : 'off'}"></div>
<input type="text" bind:value="{user.name}">
```

---

## 5. Gouvernance

Le projet Nhtml a atteint sa **Phase 3 : Industrialisation**. L'outil est fiable sur des CMS complexes, gère la navigation dynamique, maintient les liens hypertextes auto-bindés et protège formellement les chaînes de compilation.
Les prochaines étapes concernent le Server Side Rendering (SSR) et la couverture de test.

---
*Nhtml est un projet Open Source. Licence MIT — 2026.*
