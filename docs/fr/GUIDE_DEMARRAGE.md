# 🚀 Guide de Démarrage NHTML (v0.6.0)
**Bienvenue dans le Web sans JavaScript. Performance Native, Logique Serveur.**

Ce guide unique vous accompagne de l'installation à votre première interaction réactive.

---

## 1. 🏛️ Choisir son Architecture

NHTML s'adapte à votre infrastructure :

| Mode | Usage | Infrastructure | Status |
|:---|:---|:---|:---|
| **Dédié (FastCGI)** | **Recommandé**. Haute performance. | VPS / Dédié (PHP-FPM) | 🟢 Opérationnel |
| **Dédié (Rust CGI)** | Simplicité de développement. | VPS / Dédié (PHP CLI) | 🟢 Opérationnel |
| **WASM (Zéro-Serveur)** | 100% Client-side. | GitHub Pages / Statique | 🟢 Opérationnel |

---

## 2. ⚙️ Installation Rapide (Mode Dédié)

C'est la méthode la plus performante. Le **Gateway Rust** gère les WebSockets et supervise votre code PHP.

### Étape 1 : Lancer le Gateway
Téléchargez le binaire `nhtml` et lancez-le à la racine de votre projet.

**Mode Standard (CGI) :**
```bash
nhtml start --dev
```

**Mode Haute Performance (FastCGI) :**
Si vous avez un pool PHP-FPM qui tourne (ex: sur le port 9000) :
```bash
nhtml start --dev --fpm 127.0.0.1:9000
```

### Étape 2 : Structure du Projet
Placez vos fichiers dans un dossier (ex: `/mon-app`) :
- `index.nhtml` : Votre interface.
- `app.php` : Votre logique métier.

---

## 3. 🛡️ Sécurité Intégrée (v0.5.0+)

NHTML v0.5.0 introduit la **Sécurité Industrielle** par défaut :
- **HMAC-SHA256** : Tous les événements client sont signés cryptographiquement. Le Gateway rejette toute trame non authentique.
- **Sequence IDs** : Chaque action est numérotée. Les attaques par rejeu (replay) sont bloquées nativement.

---

## 4. 📡 Collaboration Temps Réel (v0.6.0+)

Vous pouvez désormais synchroniser plusieurs utilisateurs instantanément.
Dans votre `app.php`, vous pouvez demander au Gateway de diffuser un patch :
```php
// Envoie le message à TOUS les autres utilisateurs de la session
Nhtml::broadcast('others')
     ->setText('notif', "Un nouvel utilisateur a rejoint !")
     ->send();
```

---

## 5. 🐘 Votre Premier Composant

### Côté Client (`index.nhtml`)
```html
<h1 n-id="titre">Bonjour</h1>
<button n-click="btn_hello">Clique-moi</button>
```

### Côté Serveur (`app.php`)
```php
<?php
use Nhtml\Nhtml;

if ($event === 'click' && $nodeId === 'btn_hello') {
    Nhtml::patch()
         ->setText('titre', "C'est magique !")
         ->send();
}
```

---

## 📊 Outils de Diagnostic
- **Dashboard DevTools** : Rendez-vous sur `http://127.0.0.1:8081` pour voir transiter vos paquets signés en temps réel.
- **Console F12** : Vos `error_log()` PHP apparaissent directement dans la console du navigateur.
