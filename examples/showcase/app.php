<?php
/**
 * NHTML Showcase MVC - Industrial Implementation
 * Managed by PHP with local SQLite persistence
 */

// 1. Initialisation du contexte Gateway
$input = file_get_contents('php://stdin');
$ctx = json_decode($input, true);
$handler = $ctx['handler'] ?? '';
$payload = json_decode($ctx['payload'] ?? '{}', true);

// 2. Initialisation de la BDD Locale (Showcase)
$dbFile = (is_dir('/persist') || file_exists('/persist')) ? '/persist/showcase.db' : __DIR__ . '/showcase.db';
$db = new PDO("sqlite:$dbFile");
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Création des tables si inexistantes
$db->exec("CREATE TABLE IF NOT EXISTS state (key TEXT PRIMARY KEY, val TEXT)");
$db->exec("CREATE TABLE IF NOT EXISTS inventory (item TEXT PRIMARY KEY, stock INTEGER)");

// Seed initial
$initCheck = $db->query("SELECT COUNT(*) FROM state")->fetchColumn();
if ($initCheck == 0) {
    $db->exec("INSERT INTO state (key, val) VALUES ('counter', '0'), ('username', 'NemStudio'), ('email', 'contact@nhtml.io'), ('theme', 'dark'), ('lang', 'fr')");
    $db->exec("INSERT INTO inventory (item, stock) VALUES ('gateway', 12), ('sdk', 8)");
}

/**
 * Helpers State
 */
if (!function_exists('get_db_val')) {
    function get_db_val($key, $default = '') {
        global $db;
        $stmt = $db->prepare("SELECT val FROM state WHERE key = ?");
        $stmt->execute([$key]);
        $res = $stmt->fetchColumn();
        return $res !== false ? $res : $default;
    }
}

if (!function_exists('set_db_val')) {
    function set_db_val($key, $val) {
        global $db;
        $stmt = $db->prepare("INSERT OR REPLACE INTO state (key, val) VALUES (?, ?)");
        $stmt->execute([$key, $val]);
    }
}

if (!function_exists('get_stock')) {
    function get_stock($item) {
        global $db;
        $stmt = $db->prepare("SELECT stock FROM inventory WHERE item = ?");
        $stmt->execute([$item]);
        return (int)$stmt->fetchColumn();
    }
}

if (!function_exists('add_stock')) {
    function add_stock($item) {
        global $db;
        $stmt = $db->prepare("UPDATE inventory SET stock = stock + 1 WHERE item = ?");
        $stmt->execute([$item]);
    }
}

/**
 * Helpers Patch
 */
$PATCHES = [];
if (!function_exists('patch')) {
    function patch($nid, $op, $val = '', $extra = []) {
        global $PATCHES;
        $p = ["nid" => $nid, "op" => $op, "val" => $val];
        foreach($extra as $k => $v) $p[$k] = $v;
        $PATCHES[] = $p;
    }
}

if (!function_exists('log_activity')) {
    function log_activity($msg) {
        $time = date('H:i:s');
        patch('log_stream', 'append_html', "<div class='log-entry'><span class='time'>$time</span> <span class='action'>$msg</span></div>");
    }
}

if (!function_exists('switch_view')) {
    function switch_view($view) {
        $views = ['dashboard', 'messenger', 'inventory', 'settings'];
        foreach ($views as $v) {
            $active = ($v === $view);
            if ($active) {
                patch("view_$v", 'add_class', 'active');
                patch("nav_$v", 'add_class', 'active');
            } else {
                patch("view_$v", 'del_class', 'active');
                patch("nav_$v", 'del_class', 'active');
            }
        }
    }
}

// 3. Logique Applicative
$lang = get_db_val('lang', 'fr');

if (!function_exists('refresh_ui')) {
    function refresh_ui($l) {
        global $ctx;
        $isWasm = ($ctx['transport'] ?? '') === 'WASM';
        
        $trans = [
            'fr' => [
                'title' => '📊 Dashboard de Démo',
                'desc' => 'Bienvenue dans l\'écosystème NHTML. Explorez la réactivité binaire ultra-rapide.',
                'stock_title' => '📦 Gestion des Stocks',
                'messenger_title' => '💬 NHTML Messenger',
                'wasm_notice' => '🚀 <b>Mode Zero-Server Actif</b> : Vos données sont persistées localement dans votre navigateur.'
            ],
            'en' => [
                'title' => '📊 Demo Dashboard',
                'desc' => 'Welcome to the NHTML ecosystem. Explore ultra-fast binary reactivity.',
                'stock_title' => '📦 Inventory Management',
                'messenger_title' => '💬 NHTML Messenger',
                'wasm_notice' => '🚀 <b>Zero-Server Mode Active</b> : Your data is persisted locally in your browser.'
            ]
        ];
        $t = $trans[$l] ?? $trans['fr'];
        patch('page_title', 'set_text', $t['title']);
        patch('page_desc', 'set_text', $t['desc']);
        
        if ($isWasm) {
            patch('wasm_notice_box', 'set_html', $t['wasm_notice']);
            patch('wasm_notice_box', 'add_class', 'active');
        }
    }
}

if ($handler === 'init') {
    switch_view('dashboard');
    refresh_ui($lang);
    patch('counter_val', 'set_text', get_db_val('counter', '0'));
    patch('stock_gateway', 'set_text', (string)get_stock('gateway'));
    patch('stock_sdk', 'set_text', (string)get_stock('sdk'));
    patch('set_user', 'set_attr', '', ['key' => 'value', 'val' => get_db_val('username')]);
    patch('set_email', 'set_attr', '', ['key' => 'value', 'val' => get_db_val('email')]);
} else {
    // --- LANG ---
    if (strpos($handler, 'set_lang:') === 0) {
        $l = str_replace('set_lang:', '', $handler);
        set_db_val('lang', $l);
        refresh_ui($l);
        log_activity("Language set to <b>$l</b>");
    }

    // --- NAVIGATION ---
    if (strpos($handler, 'view:') === 0) {
        $view = str_replace('view:', '', $handler);
        switch_view($view);
        $titles = ['dashboard' => '📊 Dashboard', 'messenger' => '💬 Messenger', 'inventory' => '📦 Stocks', 'settings'  => '⚙️ Profil'];
        patch('page_title', 'set_text', $titles[$view] ?? 'NHTML');
        log_activity("Passage à la vue <b>$view</b>");
    }

    // --- COMPTEUR ---
    if ($handler === 'increment') {
        $val = (int)get_db_val('counter') + 1;
        set_db_val('counter', (string)$val);
        patch('counter_val', 'set_text', (string)$val);
        log_activity("Compteur incrémenté : <b>$val</b>");
    }
    if ($handler === 'decrement') {
        $val = (int)get_db_val('counter') - 1;
        set_db_val('counter', (string)$val);
        patch('counter_val', 'set_text', (string)$val);
        log_activity("Compteur décrémenté : <b>$val</b>");
    }

    // --- STYLE ---
    if ($handler === 'slider_radius') {
        $val = $payload['slider_radius'] ?? '12';
        patch('preview_mini', 'set_style', '', ['prop' => 'borderRadius', 'val' => $val . 'px']);
    }
    if ($handler === 'slider_scale') {
        $scale = (int)($payload['slider_scale'] ?? 100) / 100;
        patch('preview_mini', 'set_style', '', ['prop' => 'transform', 'val' => "scale($scale)"]);
    }

    // --- MESSENGER ---
    if ($handler === 'chat_send' || $handler === 'chat_keydown') {
        if ($handler === 'chat_keydown' && ($payload['event_key'] ?? '') !== 'Enter') { /* ignore */ }
        else {
            $msg = $payload['chat_input'] ?? '';
            if (trim($msg)) {
                patch('chat_box', 'append_html', '<div class="chat-msg sent">' . htmlspecialchars($msg) . '</div>');
                patch('chat_input', 'set_text', ''); 
                log_activity("Chat : " . htmlspecialchars(substr($msg,0,15)) . "...");
                patch('chat_box', 'append_html', '<div class="chat-msg received">Message persisté en BDD locale !</div>');
            }
        }
    }

    // --- INVENTORY ---
    if (strpos($handler, 'stock_add:') === 0) {
        $item = str_replace('stock_add:', '', $handler);
        add_stock($item);
        $new = get_stock($item);
        patch("stock_$item", 'set_text', (string)$new);
        log_activity("Réapprovisionnement : <b>$item</b> (+1 = $new)");
    }

    // --- SETTINGS ---
    if ($handler === 'save_settings') {
        $user = $payload['set_user'] ?? 'Admin';
        $email = $payload['set_email'] ?? '';
        set_db_val('username', $user);
        set_db_val('email', $email);
        patch('page_title', 'set_text', "✅ Profil sauvé : $user");
        log_activity("Configuration mise à jour pour $user");
    }

    if ($handler === 'clear_logs') {
        patch('log_stream', 'set_text', '');
    }
}

// 4. Réponse JSON
header('Content-Type: application/json');
echo json_encode(["patch" => $PATCHES]);
