# Tutoriel d'Installation & Déploiement : Nhtml V2

Ce guide vous explique comment intégrer Nhtml dans vos projets. Le moteur Nhtml V2 peut être utilisé avec ou sans PHP.

---

## ⚡ 1. Déploiement "Zéro-Bridge" (Exécution Serveur Directe)
C'est le mode le plus rapide : le serveur web appelle directement le binaire Nhtml sans passer par PHP.

### A. Sur Apache (via .htaccess)
Vous pouvez configurer Apache pour qu'il traite tous les fichiers `.nhtml` via le binaire.

```apache
# .htaccess
Options +ExecCGI
AddHandler nhtml-handler .nhtml
Action nhtml-handler /cgi-bin/nhtml.exe --cgi
```

### B. Sur Nginx (via fcgiwrap)
Nginx ne gérant pas le CGI nativement, on utilise `fcgiwrap`.

```nginx
# nginx.conf
location ~ \.nhtml$ {
    include fastcgi_params;
    fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    fastcgi_pass unix:/var/run/fcgiwrap.socket;
    # On définit l'exécuteur
    fastcgi_param FCGI_HANDLER /usr/local/bin/nhtml;
}
```

---

## 🏗️ 2. Intégration PHP (Mode Adaptateur)
Idéal si vous avez déjà un site en PHP et que vous voulez utiliser Nhtml comme moteur de rendu.

```php
require_once 'nhtml_engine/NhtmlCompiler.php';
$result = Nhtml\NhtmlCompiler::compile(file_get_contents('index.nhtml'));
echo $result['html'];
```

---

## 🛸 3. Mode "Navigateur Pur" (Bootstrapper JS)
Vous pouvez aussi laisser le navigateur lire vos fichiers `.nhtml` directement (parfait pour le développement local ou les SPA).

### Utilisation dans votre HTML :
```html
<!-- Charge le moteur WASM et scanne la page -->
<script type="module" src="examples/nhtml.js"></script>

<!-- Le navigateur "lit" ce fichier et l'affiche dans le body -->
<script type="text/nhtml" src="app.nhtml" data-target="#app"></script>

<div id="app">Chargement...</div>
```

---

## 🚀 Lequel choisir ?

| Caractéristique | Direct Serve (CGI) | PHP Bridge | Navigateur (WASM) |
| :--- | :--- | :--- | :--- |
| **Vitesse** | ⚡⚡⚡ (Max) | ⚡⚡ (Très rapide) | ⚡ (Dépend du client) |
| **SEO** | Parfait | Parfait | Moyen (besoin de JS) |
| **Installation** | Moyenne (Config serveur) | Simple | Ultra-Simple |
| **Idéal pour** | Gros trafic, API, Microservices | CMS, Blogs existants | PWA, Dashboard, Dev |

---
© 2026 NemStudio — [RETOUR AU README](./README.md)
