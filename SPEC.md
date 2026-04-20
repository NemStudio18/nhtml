# Nhtml Specification v2.0 (Headless / Manifest)

## 1. Philosophie
Nhtml v2.0 sépare strictement la **Structure (HTML)**, l'**État (State)** et la **Logique (OpCodes)**. 
- Le HTML généré est pur et "muet", utilisant des IDs techniques (`n_x`).
- Toutes les instructions sont extraites dans un manifeste JSON (`window._nhtmlAST`).
- Le moteur (JS ou Wasm) orchestre le DOM en suivant les opcodes.

### État Actuel (Phase 4 : Transition Rust)
- **Moteur Python V2** : Stable et utilisé comme spécification de référence.
- **Cœur Rust (`nhtml-core`)** : Opérationnel. Parser natif (nom) validé par tests.
- **Architecture** : Headless, Manifest JSON, Hydratation SSR.

## 2. Le Manifeste JSON (AST)
L'objet `_nhtmlAST` est injecté en haut du fichier HTML.

```json
{
  "state_vars": {
    "counter": {"initial_value": 0, "persist": false}
  },
  "nodes": {
    "n_1": {
      "type": "text",
      "expr": "counter"
    }
  }
}
```

### 2.1 Types de Nœuds
| Type | Description |
| :--- | :--- |
| `text` | Interpolation de texte simple. |
| `html` | Injection de HTML brut via l'attribut `html="{...}"`. |
| `attrs` | Liaison d'attributs et d'événements (OpCodes). Supporte le **Deep Binding** (ex: `post.title`). |
| `if` | Bloc conditionnel réactif. |
| `each` | Boucle itérative avec template et filtrage JS (`filter`). |

## 3. Implémentations

### 3.1 Prototype Python (`nhtml_engine`)
Utilisé pour le développement rapide des fonctionnalités et la validation des concepts V2.

### 3.2 Cœur Natif Rust (`nhtml-core`)
Implémentation de référence pour la performance.
- **Parser** : Basé sur les combinateurs `nom`.
- **AST** : Structures `Manifest` et `Node` typées.
- **Performance** : Compilation récursive haute performance.

## 4. Persistance
Le moteur V2 supporte la persistance automatique via l'attribut `persist="true"` sur les balises `<var>`. Les données sont synchronisées en temps réel avec le `localStorage`.
Plutôt que d'injecter du JS arbitraire, les événements sont compilés en structures de données :
- `{"op": "set", "target": "var", "value": "expr"}`
- `{"op": "increment", "target": "var", "value": 1}`
- `{"op": "call", "target": "fn", "args": [...]}`

## 5. Hydratation
1. **Initialisation** : Le moteur lit `state_vars`, crée un Proxy réactif.
2. **First Paint** : Parcourir `nodes` et appliquer les valeurs initiales.
3. **Réaction** : Toute mutation sur le Proxy déclenche `_nhtmlHydrate()` qui met à jour uniquement les nœuds dont les expressions ont changé.
