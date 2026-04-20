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

# 🇫🇷 Installation Navigateur (WASM)

Exécutez Nhtml directement dans le navigateur du visiteur.

## 1. Inclure le Bootstrapper
```html
<script type="module" src="nhtml.js"></script>
```

## 2. Utilisation des fichiers .nhtml
```html
<script type="text/nhtml" src="app.nhtml" data-target="#app"></script>
```

## 3. Configuration Mime-Type
Assurez-vous que votre serveur sert les fichiers `.wasm` avec le type `application/wasm`.
