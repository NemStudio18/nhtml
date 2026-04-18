# NHTML — Next HyperText Markup Language
## Specification v1.0 — Document de Référence de la Version Stable

**Version** : 1.0.0  
**Statut** : Stable (Production Ready / NCMS Certified)  
**Licence** : MIT  
**Date** : 17 Avril 2026

---

## 1. Introduction & Vision

Nhtml est un meta-langage déclaratif qui étend le standard HTML pour lui conférer des capacités réactives natives, sans dépendre de frameworks lourds ou de systèmes de "Virtual DOM" opaques. 

**Version 1.0 Highlights :**
- **Robustesse Industrielle** : Un parseur "Stateful" capable de gérer des expressions JavaScript complexes directement dans les attributs.
- **Performance Critique** : Latence d'initialisation proche de zéro grâce au système dual-pass (Serveur -> Client).
- **Zéro-Configuration** : Utilisation d'un moteur Python unique pour la transpilation et d'une librairie Client de moins de 5KB.

---

## 2. Nouveautés Majeures de la v1.0

La v1.0 stabilise l'écosystème après la validation terrain effectuée sur le **NCMS**.

### 2.1 Parseur d'Attributs "Context-Aware"
L'innovation majeure de la v1.0 est le moteur de tokenisation des balises. Contrairement aux versions précédentes, le parseur respecte désormais les opérateurs logiques (`>`, `<`, `=>`, `&&`) présents à l'intérieur des attributs, empêchant toute corruption de la structure HTML lors de la transpilation.

### 2.2 Scoping Isolé des Boucles (`UID Scoping`)
Les variables de boucle (ex: `as="item"`) sont désormais isolées via un système d'identifiant unique (UID). Cela permet une imbrication infinie de boucles sans risque de collision de noms de variables (Variable Shadowing), garantissant une intégrité totale des données pour des listes complexes (ex: menus récursifs).

### 2.3 Sécurité des Expressions JS (Quote Shielding)
Le moteur de synthèse JavaScript échappe automatiquement les guillemets simples à l'intérieur des expressions passées à `_nhtmlEval`. 
*Exemple supporté nativement* : `<each in="{posts.filter(p => p.type === 'post')}">`.

### 2.4 État Masqué par Défaut (Anti-Flicker)
Pour éviter le "Flicker" (affichage fugace d'un élément avant son traitement JS), toutes les balises utilisant `if` ou `visible` reçoivent automatiquement un `style="display:none"` lors de la compilation côté serveur.

---

## 3. Architecture Technique

### 3.1 Transpilation Dual-Pass
1.  **Passage 1 (Python)** : Analyse structurelle, résolution des composants, scoping des boucles, et protection des nœuds de texte.
2.  **Passage 2 (JavaScript Proxy)** : Liaison dynamique de l'état `nhtml` aux éléments `data-nhtml-*` du DOM.

### 3.2 Système de Réactivité
Nhtml v1.0 utilise un **Proxy JavaScript natif** pour intercepter les modifications de variables.
```javascript
// Tout changement ici déclenche une mise à jour ciblée du DOM
nhtml.site_name = "Nouveau Nom"; 
```
Le système utilise des `CustomEvents` pour synchroniser les groupes conditionnels et les listes dynamiques, minimisant ainsi les reflows du navigateur.

---

## 4. Guide des Balises & Attributs

### 4.1 Logique de Scripting
- `<var name=value [persist]/>` : Définit une variable d'état.
- `<if condition="...">` : Rendu conditionnel.
- `<each in="..." as="..." [filter="..."]>` : Boucle réactive.

### 4.2 Attributs Réactifs
| Attribut | Usage |
| :--- | :--- |
| `text="{expr}"` | Met à jour le `.textContent`. |
| `html="{expr}"` | Met à jour le `.innerHTML` (non échappé). |
| `visible="{expr}"` | Alias réactif pour le style display. |
| `bind:value="{var}"`| Liaison bidirectionnelle pour les inputs. |
| `{attr}="{expr}"` | Interpolation dans n'importe quel attribut standard. |

---

## 5. Intégrations CMS (NCMS Standard)

NCMS utilise Nhtml 1.0 pour :
1.  **Menus Dynamiques** : Arborescence JSON récursive.
2.  **Dashboard Admin** : Gestion de vue via `current_view`.
3.  **Éditeurs Enrichis** : Synchronisation de `Pell` avec le state Nhtml.

---

## 6. Gouvernance & Roadmap

Nhtml est entré dans sa phase de **Stabilité Long Terme**.
**Roadmap 2026-2027 :**
- **Ncss** : Extension du système de transpilation pour un CSS réactif et "scopé".
- **Nhtml v2** : Introduction d'un Virtual DOM hybride pour la manipulation de très gros volumes de données.

---
*Nhtml est un moteur Open Source propulsé par Antigravity. Licence MIT — 2026.*
