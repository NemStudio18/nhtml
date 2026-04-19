# NHTML — Next HyperText Markup Language
## Spécification v1.1 — Document de Référence Stable

**Version** : 1.1.2  
**Statut** : Stable (Production Ready / NCMS Certified)  
**Licence** : MIT  
**Date** : 19 Avril 2026  
**Certifié** : NCMS Admin Panel (CRUD complet), Public Blog, Formulaires de contact

---

## 1. Introduction & Vision

Nhtml est un meta-langage déclaratif qui étend le standard HTML pour lui conférer des capacités réactives natives, sans dépendre de frameworks lourds ou de systèmes de "Virtual DOM" opaques.

Le principe fondamental est le **Dual-Pass** : un transpileur Python côté serveur produit du HTML annoté (`data-nhtml-*`), qu'un micro-runtime JavaScript (<5KB) "réveille" dans le navigateur via un Proxy natif.

**Nouveautés v1.1 :**
- **Algorithme Split-Tag** : Isolation parfaite des nœuds texte, zéro corruption d'attributs.
- **Attributs dynamiques standards** : `class="{expr}"`, `value="{expr}"` désormais supportés.
- **Runtime garanti** : L'initialisation `window._nhtmlState` est toujours injectée, même sans variables de compilation.

---

## 2. Historique des versions

| Version | Date | Changements clés |
| :--- | :--- | :--- |
| `1.0.0` | 17 Avr 2026 | Release initiale, NCMS PoC |
| `1.0.5` | 17 Avr 2026 | Fix "Delimiter Crisis" — Regex Context-Aware |
| `1.1.0` | 18 Avr 2026 | Fix "Attribute Leakage" — Algorithme Split-Tag |
| `1.1.2` | 19 Avr 2026 | Fix split avec groupe de capture unique, runtime garanti |

---

## 3. Architecture Technique

### 3.1 Pipeline de Transpilation (parser.py)

Le parser applique les étapes suivantes dans l'ordre :

```
Source .nhtml
  ↓ 1. Suppression <!nhtml ...>
  ↓ 2. Extraction des <var>         → _nhtmlState JS
  ↓ 3. Attributs dynamiques          → data-nhtml-attrs
  ↓ 4. Titre                         → data-nhtml-text (optimisé)
  ↓ 5. Nœuds texte (Split-Tag)       → <span data-nhtml-text="...">
  ↓ 6. Composants <component>        → Enregistrement
  ↓ 7. Conditions <if>/<elseif>      → data-nhtml-if
  ↓ 8. Boucles <each>                → data-nhtml-each + JS render
  ↓ 9. Assemblage final              → HTML + CSS + JS injecté
```

### 3.2 Algorithme Split-Tag (v1.1 — correctif définitif)

L'interpolation `{expr}` dans le texte utilise un **split par balise avec un seul groupe de capture** :

```python
parts = re.split(r'(<[^>]+>)', source)
# Produit : [texte, <tag>, texte, <tag>, ...]
# Seuls les segments sans '<' sont traités pour l'interpolation
```

Cette approche garantit que les attributs HTML comme `onclick="editing_category.name='...'"` ne sont **jamais** vus par le moteur d'interpolation.

### 3.3 Système de Réactivité (Client)

Le moteur client est un Proxy JavaScript natif :

```javascript
window._nhtmlState = { site_name: "NCMS", posts: [], ... };
const nhtml = new Proxy(_nhtmlState, {
    set(target, prop, value) {
        target[prop] = value;
        _nhtmlUpdateDOM();  // Mise à jour ciblée du DOM
        document.dispatchEvent(new Event('nhtml:update'));
        return true;
    }
});
```

Les directives DOM reconnues :

| Directive | Comportement |
| :--- | :--- |
| `data-nhtml-text="expr"` | Met à jour `.textContent` |
| `data-nhtml-html="expr"` | Met à jour `.innerHTML` |
| `data-nhtml-if="expr"` | Toggle `display` selon vérité |
| `data-nhtml-each="arr"` | Re-render la liste depuis le JS blueprint |
| `data-nhtml-attrs="json"` | Met à jour les attributs (class, value, href...) |

### 3.4 Injection de l'état PHP → JS

```php
// View.php — injectState()
Object.assign(nhtml, <?php echo json_encode($data); ?>);
```

Cette ligne, injectée avant `</body>`, met à jour le Proxy et déclenche automatiquement le re-render complet de la page avec les données serveur.

---

## 4. Guide des Balises & Attributs

### 4.1 Déclaration et Variables

```html
<!nhtml 1.0>                              <!-- Déclaration optionnelle -->
<var site_name="NCMS">                    <!-- Variable scalaire -->
<var posts=[] persist>                    <!-- Tableau + persistence localStorage -->
<var editing_post={"id": null, ...}>      <!-- Objet JSON -->
```

### 4.2 Interpolation de texte

```html
<h1>{site_name} Admin</h1>
<!-- Compilé en : -->
<h1><span data-nhtml-text="{site_name}">{site_name}</span> Admin</h1>
```

### 4.3 Conditions

```html
<if condition="current_view == 'articles'">
    <div>Vue articles</div>
</if>
<elseif condition="current_view == 'pages'">
    <div>Vue pages</div>
</elseif>
<else>
    <div>Autre vue</div>
</else>
```

### 4.4 Boucles

```html
<each in="{posts.filter(p => p.type === 'post')}" as="p" tag="tbody">
    <tr>
        <td>{p.title}</td>
        <td>{p.status}</td>
    </tr>
</each>

<!-- Avec index et filtre -->
<each in="{categories}" as="cat" index="i" filter="cat.active === true">
    <option value="{cat.id}">{cat.name}</option>
</each>

<!-- Message "vide" associé -->
<empty for="categories">Aucune catégorie.</empty>
```

### 4.5 Attributs dynamiques

```html
<!-- Via valeur : attr="{expr}" -->
<a href="/admin" class="{current_view == 'home' ? 'active' : ''}">

<!-- Via clé : {attr}="valeur" -->
<div {disabled}="true">

<!-- Liaison bidirectionnelle -->
<input bind:value="{editing_post.title}" name="title">
```

### 4.6 Composants

```html
<component name="PostCard">
    <props>
        <prop name="title" default="Sans titre"/>
        <prop name="status" default="draft"/>
    </props>
    <article class="card">
        <h3>{title}</h3>
        <span class="status {status}">{status}</span>
    </article>
</component>
```

### 4.7 Titre réactif

```html
<title>Dashboard Admin — {site_name}</title>
<!-- Compilé en : -->
<title data-nhtml-text="Dashboard Admin — {site_name}">Dashboard Admin — {site_name}</title>
```

---

## 5. CLI — Interface de Compilation

```bash
# Compiler un seul fichier
python nhtml.py source.nhtml output.html --runtime=inline

# Compiler tout un dossier (mode production)
python nhtml.py build templates/ public/cache/ --runtime=inline

# Modes de runtime disponibles
--runtime=inline    # JS intégré dans le HTML (défaut, recommandé)
--runtime=cdn       # Tag <script src="cdn">
--runtime=none      # Pas de runtime (pour tests)
```

---

## 6. Intégration NCMS (Standard certifié v1.1)

### 6.1 Structure NCMS

```
NCMS/
├── templates/          # Sources .nhtml
│   ├── admin.nhtml     # Dashboard multi-vues (Articles, Pages, Catégories, Nav, Forms)
│   ├── blog.nhtml      # Liste des articles publics
│   ├── post.nhtml      # Article individuel
│   └── login.nhtml     # Page de connexion
├── public/
│   ├── cache/          # HTML compilés (générés par nhtml.py)
│   └── index.php       # Point d'entrée
└── src/
    ├── Core/           # Router, Auth, View, Database
    ├── Models/         # Post, Category, MenuLink, Form
    └── Controllers/    # AdminController, BlogController
```

### 6.2 Flux de rendu

```
Requête HTTP → index.php → Router → Controller
    → View::render('admin', $data)
        → Vérification cache (nhtml.py si périmé)
        → file_get_contents(admin.html)
        → injectState() → Object.assign(nhtml, $data)
        → echo $html
```

### 6.3 CRUD Admin certifié

Le dashboard admin (`admin.nhtml`) gère en une seule page :

| Module | Vue | Opérations |
| :--- | :--- | :--- |
| Articles | `?view=articles` | CREATE via `/admin/create`, READ liste, UPDATE via `/admin/edit/{id}`, DELETE via POST |
| Pages statiques | `?view=pages` | Idem articles, type=page |
| Catégories | `?view=categories` | Formulaire inline, CREATE/UPDATE/DELETE |
| Navigation | `?view=navigation` | Gestion des liens de menu |
| Formulaires | `?view=forms` | Création de formulaires dynamiques |
| Réponses | `?view=submissions` | Lecture des soumissions |

---

## 7. Gouvernance & Roadmap

### Roadmap 2026-2027

| Milestone | Fonctionnalité | Description |
| :--- | :--- | :--- |
| **v1.2** | Hot Reload | Recompilation automatique à la sauvegarde |
| **v1.3** | Ncss Alpha | CSS réactif et scopé. `@if (is_admin) { ... }` |
| **v1.5** | PWA Native | Service Worker auto-généré pour NCMS |
| **v2.0** | Hybrid VDom | Virtual DOM partiel pour 10 000+ lignes |

### Contraintes connues

1. **Profondeur d'état** : Les objets profondément imbriqués (>3 niveaux) peuvent ne pas être correctement réactifs. Préférer des structures plates.
2. **Support navigateur** : Nécessite `Proxy` natif (IE11 non supporté).
3. **Expressions imbriquées** : Les `{expr}` contenant des `{...}` imbriqués ne sont pas supportés.

---

*Nhtml est un moteur Open Source propulsé par Antigravity. Licence MIT — 2026.*  
*Spécification maintenue par le projet NCMS.*
