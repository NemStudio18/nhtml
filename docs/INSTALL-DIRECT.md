# Direct Server Installation (Zero-Bridge)

Run Nhtml directly from your web server without PHP or Node.js. This mode is the fastest for serving static-like files with dynamic features.

## Step 1: Binary Setup
Download the Nhtml binary for your OS and place it in a secure location (e.g., `/usr/local/bin/nhtml` or `C:\nhtml\nhtml.exe`).

## Step 2: Apache Setup (.htaccess)
Standard CGI setup to map `.nhtml` files to the compiler.

```apache
# .htaccess
Options +ExecCGI
AddHandler nhtml-handler .nhtml
Action nhtml-handler /cgi-bin/nhtml --cgi
```

## Step 3: Nginx Setup
Since Nginx doesn't support CGI natively, use `fcgiwrap`.

```nginx
location ~ \.nhtml$ {
    include fastcgi_params;
    fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    fastcgi_pass unix:/var/run/fcgiwrap.socket;
    fastcgi_param FCGI_HANDLER /usr/local/bin/nhtml;
}
```

---

# 🇫🇷 Installation Directe (Serveur)

Exécutez Nhtml directement depuis votre serveur web sans PHP ni Node.js.

## 1. Installation du Binaire
Placez le binaire `nhtml` dans un dossier sécurisé de votre serveur.

## 2. Configuration Apache
```apache
AddHandler nhtml-handler .nhtml
Action nhtml-handler /cgi-bin/nhtml --cgi
```

## 3. Configuration Nginx
Utilisez `fcgiwrap` pour faire le pont entre Nginx et le binaire.
