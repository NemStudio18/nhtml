# 🚀 Guide de Démarrage NHTML (v0.4.0)
**Bienvenue dans le Web sans JavaScript. Performance Native, Logique Serveur.**

Ce guide unique vous accompagne de l'installation à votre première interaction réactive.

---

## 1. 🏛️ Choisir son Architecture

NHTML s'adapte à votre infrastructure :

| Mode | Usage | Infrastructure | Status |
|:---|:---|:---|:---|
| **Dédié (Rust)** | **Recommandé**. Temps réel pur. | VPS / Dédié (Accès CLI) | 🟢 Opérationnel |
| **Mutualisé (HTTP)** | Fallback universel. | Hébergement standard (OVH, etc.) | 🟢 Opérationnel |
| **WASM** | 100% Client-side. | GitHub Pages / Statique | 🟠 Bientôt (Bundle PHP.wasm en préparation) |

---

## 2. ⚙️ Installation Rapide (Mode Dédié)

C'est la méthode la plus performante. Le **Gateway Rust** gère les WebSockets et supervise votre code PHP.

### Étape 1 : Lancer le Gateway
Téléchargez le binaire `gateway` et lancez-le à la racine de votre projet :
```bash
# Mode développement avec auto-rechargement (Ports par défaut : WS=8080, PHP=8000)
nhtml start --dev

# Vous pouvez personnaliser les ports :
nhtml start --dev --ws-port 9080 --php-port 9000
```
*Le Gateway lance un serveur HTTP (par défaut sur 3000) et le serveur PHP.*

### Étape 2 : Structure du Projet
Placez vos fichiers dans un dossier (ex: `/mon-app`) :
- `index.nhtml` : Votre interface.
- `app.php` : Votre logique métier.

---

## 3. 🐘 Votre Premier Composant

### Côté Client (`index.nhtml`)
Identifiez les éléments à mettre à jour avec `n-id` et capturez les clics avec `n-click`.
```html
<h1 n-id="titre">Bonjour</h1>
<button n-click="btn_hello">Clique-moi</button>

<!-- Magie ! ZÉRO balise <script>. Le Gateway injecte automatiquement le Bridge JS en développement. -->
```

### Côté Serveur (`app.php`)
Utilisez le SDK PHP pour envoyer des ordres binaires instantanés.
```php
<?php
$nhtml = GatewayFactory::create();

if ($event === 'click' && $nodeId === 'btn_hello') {
    $nhtml->setText('titre', "C'est magique !")
          ->send();
}
```

---

## 4. 🛰️ Concepts Clés pour Réussir

1.  **Le Gateway est un Facteur** : Il ne connaît pas votre métier, il se contente de livrer des paquets binaires (NBPS) entre le PHP et le navigateur.
2.  **NodeIDs** : Utilisez des IDs simples (ou numériques). Le serveur pilote le DOM à distance via ces IDs.
3.  **Zéro Latence (Local Actions)** : Pour les effets visuels (hover, scroll), utilisez les **Local Actions** (voir `SPEC.md`) pour une exécution immédiate sans aller-retour serveur.

---

## 📊 Outils de Diagnostic
- **Dashboard DevTools** : Rendez-vous sur `http://127.0.0.1:8081` pour voir transiter vos paquets en temps réel.
- **Console F12** : Vos `error_log()` PHP apparaissent directement dans la console du navigateur avec le préfixe `[NHTML SERVER]`.
