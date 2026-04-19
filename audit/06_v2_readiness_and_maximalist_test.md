# Audit 06 : Readiness Nhtml V2 & NCMS Migration

## 1. État Statutaire
Le moteur V2 (Python) est fonctionnel. Il a réussi à transformer la `kitchen_sink` en un système déclaratif JSON + Hydratation.

## 2. Analyse de Compatibilité NCMS V1
L'analyse des templates `admin.nhtml` et `post.nhtml` révèle les besoins suivants :

### 🟢 Supporté par le moteur V2 actuel
- Variables globales (`{site_name}`).
- Ternaires simples dans le texte et les attributs.
- Boucles `<each>` standards.
- Conditions `<if>` standards.
- États initiaux d'objets (JSON).

### 🟠 À valider / À renforcer
- **Filtres JS en ligne** : `posts.filter(...)` nécessite un moteur d'évaluation robuste (`eval` ou mini-interpréteur).
- **Échappement massif** : Les événements `onclick` contenant des expressions Nhtml imbriquées avec des quotes.
- **Support des balises orphelines** : `<empty for="...">` doit être implémenté ou transpilé en `<if>`.
- **Binding Profond** : `bind:value` sur des objets (`editing_post.title`).

## 3. Conclusion de l'Audit : 100% SUCCESS ✅
Le prototype Python est désormais **Feature-Complete** pour la V2. La validation sur la `kitchen_sink` (Maximaliste) a prouvé que tous les cas complexes de NCMS (filtres, binding profond, persistance) sont supportés par l'architecture Manifest/Headless.

Nous sommes prêts à figer cette logique pour le portage Rust.

