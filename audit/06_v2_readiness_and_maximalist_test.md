# Audit 06 : Readiness Nhtml V2 & NCMS Migration

## 1. État Statutaire
Le moteur V2 (Python) est fonctionnel. Il a réussi à transformer la `kitchen_sink` en un système déclaratif JSON + Hydratation.

## 2. Analyse de Compatibilité NCMS V1
L'analyse des templates `admin.nhtml` et `post.nhtml` révèle les besoins suivants :

### 🟢 Supporté par le moteur V2 actuel (Python & Rust)
- Variables globales (`{site_name}`).
- Ternaires simples dans le texte et les attributs.
- Boucles `<each>` standards.
- Conditions `<if>` standards.
- États initiaux d'objets (JSON).
- **Filtres JS en ligne** : `posts.filter(...)` géré par le fallback `Eval`.
- **Échappement massif** : Événements `onclick` gérés sans erreur grâce au parsing natif rigoureux (`parse_action`).
- **Binding Profond** : `bind:value` transpilé correctement en `Set { target: "parent.child" }`.
- **Support des balises orphelines** : `<empty for="...">` implémenté nativement et transpilé dynamiquement vers `<if>`.

## 3. Conclusion de l'Audit : 100% SUCCESS ✅
Le prototype Python est désormais **Feature-Complete** pour la V2. La validation sur la `kitchen_sink` (Maximaliste) a prouvé que tous les cas complexes de NCMS (filtres, binding profond, persistance) sont supportés par l'architecture Manifest/Headless.

Nous sommes prêts à figer cette logique pour le portage Rust.

---

## 4. Bilan du Portage Rust CLI (Phase 4.3) : Parité Atteinte ✅

Le portage du parser Python vers une architecture native Rust (CLI + Library) a été réalisé avec un niveau de parité fonctionnelle ciblé respecté.

### 📊 Rapport de Parité Python ↔ Rust

| Composant | Statut Parité | Note technique |
| :--- | :--- | :--- |
| **Variables d'état** | ✅ Totale | `<var>` parsées, typage primitif. |
| **Nœuds `text` & `attrs`** | ✅ Totale | Captures des expressions et bindings identiques. |
| **Nœuds `each`** | ✅ Totale | Expressions `expr_in` extraites proprement. |
| **Two-way Binding** | ✅ Totale | `bind:` génère automatiquement `on:input`/`on:change`. |
| **Variables JSON** | ✅ Totale | Rust parse nativement via `serde_json::from_str` à la compilation. Plus aucun flag temporaire. |
| **Opcodes complexes** | ✅ Totale | Expressions simples parsées en `Set`, `Increment`, `Call`. Sécurisé ! |

Cette base est suffisamment robuste et alignée avec Python pour procéder à la **Phase 4.4 : Test JS Bridge / WASM dans le navigateur réel**.
