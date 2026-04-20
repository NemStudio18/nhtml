# Browser Installation (WebAssembly)

Run Nhtml directly in the visitor's browser. Perfect for PWAs, Dashboards, and zero-server architectures.

## Step 1: Include the Bootstrapper
Copy `examples/nhtml.js` and the `pkg/` folder to your static directory.

```html
<script type="module" src="nhtml.js"></script>
```

## Step 2: Use .nhtml Files
You can now define your components directly in HTML or link external files.

```html
<!-- Remote file -->
<script type="text/nhtml" src="views/app.nhtml" data-target="#main-app"></script>

<!-- Inline component -->
<script type="text/nhtml" data-target="#sidebar">
    <var name="status" value="'Active'">
    <p>System Status: {status}</p>
</script>
```

## Step 3: Server Configuration
Ensure your web server serves `.wasm` files with the correct MIME type: `application/wasm`.

---

# 🇫🇷 Installation Navigateur (WebAssembly)

Exécutez Nhtml directement dans le navigateur du visiteur. Parfait pour les PWAs, les Dashboards et les architectures sans serveur.

## Étape 1 : Inclure le Bootstrapper
Copiez `examples/nhtml.js` et le dossier `pkg/` dans votre répertoire statique.

```html
<script type="module" src="nhtml.js"></script>
```

## Étape 2 : Utilisation des fichiers .nhtml
Vous pouvez maintenant définir vos composants directement dans l'HTML ou lier des fichiers externes.

```html
<!-- Fichier distant -->
<script type="text/nhtml" src="views/app.nhtml" data-target="#main-app"></script>

<!-- Composant en ligne -->
<script type="text/nhtml" data-target="#sidebar">
    <var name="status" value="'Active'">
    <p>Statut du système : {status}</p>
</script>
```

## Étape 3 : Configuration Serveur
Assurez-vous que votre serveur web sert les fichiers `.wasm` avec le type MIME correct : `application/wasm`.
