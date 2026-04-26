# ❓ Foire Aux Questions (FAQ)

Bienvenue dans la FAQ de NHTML. Si vous ne trouvez pas de réponse à votre question ici, n'hésitez pas à ouvrir une discussion sur le dépôt officiel.

---

## 🛠️ Développement & SDK

### Puis-je créer mes propres helpers par-dessus le SDK officiel ?
**Oui, absolument !** C'est même une pratique recommandée pour garder votre code propre. Comme le SDK PHP retourne des objets de mutation simples, vous pouvez étendre la classe `Patch` pour créer vos propres "combos" d'actions.

**Exemple :**
```php
class MonPatch extends Nhtml\Patch {
    public static function notifySuccess(string $message): array {
        return [
            self::setText('notification-area', $message),
            self::addClass('notification-area', 'is-success'),
            self::addClass('notification-area', 'show'),
        ];
    }
}
```

### NHTML est-il compatible avec Tailwind CSS ou Bootstrap ?
**Oui.** NHTML ne manipule que le DOM standard. Vous pouvez utiliser n'importe quel framework CSS. Les attributs `n-` (comme `n-click` ou `n-id`) n'interfèrent pas avec vos classes CSS. Vous pouvez même utiliser `Patch::addClass()` ou `Patch::toggleClass()` pour piloter vos animations Tailwind depuis le serveur.

---

## 🛰️ Protocole & Performance

### Pourquoi reste-t-il du JavaScript (`bridge.js`) ?
Le `bridge.js` de ~2KB est une **couche de transition**. Actuellement, aucun navigateur ne comprend nativement le protocole binaire NBPS. Le bridge sert d'interprète : il reçoit le binaire, le décompresse et applique les mutations au DOM. L'objectif à long terme est une intégration native (via notre fork Chromium) qui rendra ce script inutile.

### Puis-je utiliser NHTML avec un autre langage que PHP ?
**Oui.** NHTML est avant tout un **protocole** (NBPS). Tant que votre programme peut recevoir du JSON/Binaire et renvoyer des trames conformes à la `SPEC.md`, vous pouvez écrire un SDK dans n'importe quel langage (Python, Go, Node.js, etc.).

---

## 🛡️ Sécurité & Déploiement

### Comment sécuriser l'accès aux DevTools en production ?
Par défaut, les DevTools ne sont accessibles que sur `127.0.0.1`. En production, nous recommandons d'utiliser un **Tunnel SSH** pour y accéder sans ouvrir de port public. Si vous devez les exposer, utilisez impérativement le flag `--devtools-token` pour protéger l'accès par un mot de passe dans l'URL.

### Les WebSockets sont-ils obligatoires ?
Non. Bien que les WebSockets offrent les meilleures performances pour le temps réel, NHTML possède un mode de fallback automatique en **HTTP POST** pour les hébergements mutualisés classiques qui ne supportent pas les connexions persistantes.
