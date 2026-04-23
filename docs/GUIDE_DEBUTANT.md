# 🚀 Bienvenue dans NHTML (v0.2.2)
## Le Web sans JavaScript. Performance Native.

Tu es débutant ? Tu trouves que le développement web moderne est devenu une "bouillie de frameworks" illisible ? NHTML est fait pour toi. Ce document t'explique comment ça marche avec des mots simples.

---

### 1. C'est quoi le concept ?
D'habitude, pour rendre un site interactif, tu dois apprendre le **JavaScript (JS)**, React, Vue ou Angular. C'est complexe, ça change tous les mois, et ça rend les sites lourds.

**Avec NHTML, tu n'écris PLUS de JavaScript.** 
Tu écris du HTML augmenté. Le navigateur ne "réfléchit" plus, il devient un **terminal léger** qui reçoit des ordres de ton serveur.

---

### 2. Le "Avant vs Après"

#### ❌ Avant (Le chaos du JavaScript)
Pour faire un bouton qui change un texte :
1. HTML pour le bouton.
2. JS pour "écouter" le clic.
3. Fetch (réseau) vers le serveur.
4. JSON reçu, puis transformation manuelle du texte en JS.

#### ✅ Après (La simplicité NHTML v0.2.2)
Tu écris juste ça dans ton fichier HTML :
```html
<h1 data-n-id="42">Bonjour</h1>
<button n-click="changer_nom.php">Clique-moi</button>
```
Et dans ton code **PHP**, tu utilises le SDK Pro :
```php
$nhtml->setText(42, "C'est magique !")->send();
```
NHTML s'occupe de transformer ça en **paquet binaire ultra-rapide** et de mettre à jour l'écran instantanément.

---

### 3. Les 3 ingrédients magiques

1.  **Le Gateway (Le Facteur Rust)** : C'est le cœur. Un petit programme ultra-rapide (écrit en Rust) qui fait le lien entre ton code PHP et le navigateur. Il parle en **NBPS**, un langage binaire que les machines adorent.
2.  **Le Polyfill (Le Réceptionniste WASM)** : Un micro-programme caché dans le navigateur qui reçoit les ordres du Gateway et modifie ta page à une vitesse native.
3.  **Le SDK (Le Cerveau)** : Une petite bibliothèque dans ton langage préféré (ex: PHP). C'est là que tu décides de la logique : *"Si on clique ici, change la couleur là-bas"*.

---

### 4. Pourquoi c'est révolutionnaire ?

*   **Zéro JS Développeur** : 100% de ta logique est au même endroit (ton back-end).
*   **Sécurité P3** : Comme tout est contrôlé par le serveur, les pirates ne peuvent pas tricher avec le code du navigateur.
*   **Local Actions (0ms)** : Pour les effets visuels (survol, défilement), NHTML exécute tout directement dans le navigateur. Pas d'attente, pas de latence.

---

**Prêt à révolutionner ton workflow ?** 
Lance le Gateway, installe le SDK PHP et regarde ton interface prendre vie sans une seule ligne de JS ! 🛰️🧤🚀
