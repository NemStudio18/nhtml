# NHTML — Next HyperText Markup Language
## Specification v0.8 — Document de Référence

**Version** : 0.8.1  
**Statut** : Release Candidate (Agnostic, Logging, & Native HTML)  
**Licence** : MIT  
**Date** : 17 Avril 2026

---

## 1. Introduction & Vision

Nhtml est un langage déclaratif qui étend le HTML standard pour lui donner des capacités réactives natives sans dépendre de frameworks complexes (React, Vue, etc.). 

**Principes Clés :**
- **Un seul fichier** : Logique, style et structure cohabitent par composant.
- **Transpilation Headless** : Le code Nhtml est traduit en HTML/JS pur compréhensible par n'importe quel navigateur.
- **Agnostique et Autonome** : Nhtml ne dépend d'aucun backend (PHP, Node, etc.). Le moteur Python interne gère de bout en bout la compilation, l'observation des fichiers (watch), et l'exécution client avec un système de logging intégré.

---

## 2. Nouveautés de la v0.8 (Fiabilité & Indépendance)

La version 0.8 et 0.8.1 propulsent Nhtml vers une robustesse de production. L'accent est mis sur l'évaluation sécurisée du JavaScript, la gestion fine du CSS, et l'injection HTML.

### 2.1 Mode Watch Autonome & Logging
Le script `nhtml.py` devient un CLI robuste avec journalisation d'événements :
- `python nhtml.py build <dir> <out_dir>` : Génération statique totale (`SSG`).
- `python nhtml.py watch <dir> <out_dir>` : Boucle de conception en temps réel s'exécutant de manière silencieuse et résiliente.
- **Logging Python** : Les statuts de compilation et les erreurs fatales sont archivés dans `nhtml.log` pour garder la console propre. 
- Les erreurs d'évaluation JavaScript sur le navigateur (`_nhtmlEval`) ne brisent plus l'application mais sont interceptées nativement dans `console.warn`.

### 2.2 Rendu HTML brut interactif (`html="..."`)
Nhtml introduit un attribut `html=` permettant d'injecter nativement le rendu d'un éditeur de texte enrichi (ex: Pell) sans échapper le contenu (utilisation de `innerHTML` côté proxy Javascript).
```html
<var my_rich_text="<p>Ceci est <strong>Gras</strong></p>">

<!-- Contenu protégé (échappé) -->
<div text="{my_rich_text}"></div>

<!-- Contenu HTML interprété Nativement par Nhtml -->
<div html="{my_rich_text}"></div>
```

### 2.3 Protection des Ternaires et Scripts Externes
- **JavaScript Intelligent** : L'évaluateur Nhtml comprend désormais la différence absolue entre un fragment statique JSON (`{id: 1}`) et un opérateur conditionnel ternaire JavaScript complexe de rendu `{view == 'edit' ? 'Editer' : 'Nouveau'}`. 
- La balise `<script src="...">` est préservée et re-packagée proprement dans le Header final lors de l'assemblage.

### 2.4 Composants CSS Intégrés
Nhtml excelle par la déclaration d'un `<style>` embarqué à la fin de chaque page `.nhtml` ou composant `<component>`. Le parseur extrait, condense, et injecte le tout de manière optimisée via l'orchestrateur.

---

## 3. Structure d'un fichier Nhtml

Un fichier Nhtml peut contenir les blocs correspondants au standard :

| Bloc | Rôle |
| :--- | :--- |
| `<head>` | Métadonnées et liens (scripts externes préservés, styles fusionnés) |
| `<var>` | Déclaration de variables d'état (attribut `persist` optionnel via LS) |
| `<body>` | Structure de la page |
| `<if>` / `<elseif>` / `<else>` | Logique conditionnelle native paramétrique |
| `<each in= as=>` | Scope iterator via Proxy |
| `<style>` | CSS encapsulé, récolté par transpilation logicielle |
| `<script>` | Logique JS injectée post-assemblage |

---

## 4. Syntaxe de Réactivité

### 4.1 Variables globales
```html
<var site_name="NCMS">
<!-- Réactivité bidirectionnelle native (attributs bind:) -->
<input type="text" bind:value="{site_name}">
```

### 4.2 Restitution & Interpolation UI
```html
<h1 text="{site_name}"></h1>
<div html="{content}"></div>
<small class="tag">{tag_name || ''}</small>
```

---

## 5. Gouvernance

Le projet Nhtml a complété sa **Phase 3 : Industrialisation** (avec le NCMS). L'outil est devenu sa propre "Toolchain" de Build, affranchissant tous les développeurs Webs de tout système lourd, en garantissant un pur Front-End agnostic.

---
*Nhtml est un moteur Open Source propulsé par Antigravity. Licence MIT — 2026.*
