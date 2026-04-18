# NHTML — Next HyperText Markup Language
## Specification v0.3 — Document de Référence

**Version** : 0.3.0  
**Statut** : Version de travail (Bêta)  
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

## 2. Nouveautés de la v0.3

La version 0.3 apporte des améliorations majeures issues de retours d'utilisation réelle (CMS) :

### 2.1 Attribut `tag` pour les boucles `<each>`
Il est désormais possible de spécifier la balise HTML qui servira de conteneur pour une boucle. Cela permet une intégration parfaite dans les structures strictes comme les tableaux (`<table>`).
```html
<each in="{posts}" as="p" tag="tbody">
    <tr><td>{p.title}</td></tr>
</each>
```

### 2.2 Interpolation Intelligente
L'interpolation `{expression}` utilise désormais une détection par expressions régulières (Regex) précise. Cela évite de corrompre les objets JSON ou les blocs de code JavaScript internes, permettant une cohabitation fluide entre Nhtml et JS classique.

### 2.3 Réactivité native du Titre
La balise `<title>` est traitée spécifiquement pour rester réactive sans injection de balises `<span>`. L'onglet du navigateur se met à jour dynamiquement tout en restant valide syntaxiquement.

### 2.4 Liaison de données par Propriétés (.value)
Pour les formulaires (`input`, `select`, `textarea`), la réactivité passe désormais par la propriété `.value` de l'élément DOM au lieu de l'attribut HTML `value`. Cela garantit que les champs sont correctement pré-remplis même après une interaction utilisateur.

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

Le projet Nhtml progresse vers la **Phase 2 : Adoption**. Toute évolution du langage doit être documentée et testée par rapport à la rétro-compatibilité HTML5.

---
*Nhtml est un projet Open Source. Licence MIT — 2026.*
