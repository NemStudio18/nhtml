# ❓ Frequently Asked Questions (FAQ)

Welcome to the NHTML FAQ. If you don't find the answer you're looking for, feel free to open a discussion on the official repository.

---

## 🛠️ Development & SDK

### Can I create my own helpers on top of the official SDK?
**Yes, absolutely!** This is a recommended practice to keep your code clean. Since the PHP SDK returns simple mutation objects, you can extend the `Patch` class to create your own "combos" of actions.

**Example:**
```php
class MyPatch extends Nhtml\Patch {
    public static function notifySuccess(string $message): array {
        return [
            self::setText('notification-area', $message),
            self::addClass('notification-area', 'is-success'),
            self::addClass('notification-area', 'show'),
        ];
    }
}
```

### Is NHTML compatible with Tailwind CSS or Bootstrap?
**Yes.** NHTML only manipulates the standard DOM. You can use any CSS framework. The `n-` attributes (like `n-click` or `n-id`) do not interfere with your CSS classes. You can even use `Patch::addClass()` or `Patch::toggleClass()` to drive your Tailwind animations from the server.

---

## 🛰️ Protocol & Performance

### Why is there still JavaScript (`bridge.js`)?
The `bridge.js` script (~25KB) is a **transition layer**. Currently, no browser natively understands the binary NBPS protocol. The bridge acts as an interpreter: it receives the binary, decompresses it, and applies the mutations to the DOM. The long-term goal is native integration (via our Chromium fork) which will make this script unnecessary.

### Can I use NHTML with a language other than PHP?
**Yes.** NHTML is primarily a **protocol** (NBPS). As long as your program can receive JSON/Binary and return frames compliant with `SPEC.md`, you can write a SDK in any language (Python, Go, Node.js, etc.).

---

## 🛡️ Security & Deployment

### How do I secure DevTools access in production?
By default, DevTools are only accessible on `127.0.0.1`. In production, we recommend using an **SSH Tunnel** to access them without opening a public port. If you must expose them, you must use the `--devtools-token` flag to protect access with a password in the URL.

### Are WebSockets mandatory?
No. While WebSockets offer the best performance for real-time, NHTML has an automatic fallback mode using **HTTP POST** for shared hosting environments that do not support persistent connections.
