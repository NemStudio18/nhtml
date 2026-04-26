<?php
/**
 * NHTML v0.4.0 - Démo Chat (Industrial Version)
 */

require_once __DIR__ . '/../../sdk/php/src/Nhtml.php';
require_once __DIR__ . '/../../sdk/php/src/Patch.php';

use Nhtml\Nhtml;

// --- Initialisation BDD Chat ---
$db = new PDO('sqlite:' . __DIR__ . '/chat.db');
$db->exec("CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT,
    author TEXT,
    content TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
)");
$db->exec("CREATE TABLE IF NOT EXISTS users (session_id TEXT PRIMARY KEY, pseudo TEXT)");

// --- Lecture du contexte via STDIN ---
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$formData = json_decode($input['payload'] ?? '{}', true);
$sessionId = $input['session_id'] ?? 'anonymous';

// Gérer le pseudo (Sauvegarde automatique si présent dans le payload)
$pseudo = trim($formData['chat_pseudo'] ?? '');
if ($pseudo) {
    $stmt = $db->prepare("INSERT INTO users (session_id, pseudo) VALUES (?, ?) ON CONFLICT(session_id) DO UPDATE SET pseudo = excluded.pseudo");
    $stmt->execute([$sessionId, $pseudo]);
} else {
    $stmt = $db->prepare("SELECT pseudo FROM users WHERE session_id = ?");
    $stmt->execute([$sessionId]);
    $pseudo = $stmt->fetchColumn() ?: 'Anonyme';
}

$p = Nhtml::patch();

// Si c'est l'init (via HELLO), on charge les derniers messages
if ($handler === 'init' || !$handler) {
    $stmt = $db->query("SELECT * FROM messages ORDER BY id DESC LIMIT 20");
    $messages = array_reverse($stmt->fetchAll(PDO::FETCH_ASSOC));
    
    $html = "";
    foreach ($messages as $msg) {
        $isMe = $msg['session_id'] === $sessionId;
        $class = $isMe ? 'sent' : 'received';
        
        // Chercher le pseudo de l'auteur
        $stmt_u = $db->prepare("SELECT pseudo FROM users WHERE session_id = ?");
        $stmt_u->execute([$msg['session_id']]);
        $author = $stmt_u->fetchColumn() ?: substr($msg['session_id'], 0, 5);
        
        if ($isMe) $author = "Moi ($author)";
        
        $html .= "<div class='message $class'><div class='author'>$author</div>{$msg['content']}</div>";
    }
    
    if ($html) $p->replaceInner('msg_list', $html);
    $p->setAttr('chat_pseudo', 'value', $pseudo);
}

// Envoi d'un message
if ($handler === 'send') {
    $eventKey = $formData['event_key'] ?? '';
    // On n'envoie que si c'est un clic sur bouton (pas de touche) ou la touche 'Enter'
    if ($eventKey !== '' && $eventKey !== 'Enter') {
        $p->send(); // On ne fait rien d'autre
        exit;
    }
    
    $content = trim($formData['chat_input'] ?? '');
    if ($content !== '') {
        $stmt = $db->prepare("INSERT INTO messages (session_id, author, content) VALUES (?, ?, ?)");
        $stmt->execute([$sessionId, $pseudo, $content]);
        
        $p->appendHtml('msg_list', "
            <div class='message received'>
                <div class='author'>$pseudo</div>
                $content
            </div>
        ")->broadcast()->setText('chat_input', '')->focus('chat_input')->scrollTo('msg_list');
    }
}

$p->send();
