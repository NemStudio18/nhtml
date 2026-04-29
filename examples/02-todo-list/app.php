<?php
/**
 * NHTML v0.4.0 - Démo ToDo List (Industrial SQLite Version)
 */

require_once __DIR__ . '/../../sdk/php/src/Nhtml.php';
require_once __DIR__ . '/../../sdk/php/src/Patch.php';

use Nhtml\Nhtml;

// --- Initialisation SQLite ---
try {
    $db = new PDO('sqlite:' . __DIR__ . '/todo.db');
    $db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    $db->exec("CREATE TABLE IF NOT EXISTS tasks (
        id TEXT PRIMARY KEY,
        text TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )");
} catch (PDOException $e) {
    echo json_encode([['op' => 'set_text', 'nid' => 'todo_list', 'val' => 'Erreur BDD: ' . $e->getMessage()]]);
    exit;
}

// --- Lecture via STDIN ---
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$formData = json_decode($input['payload'] ?? '{}', true);

$p = Nhtml::patch();

// On ne traite l'initialisation que si le handler est explicitement 'init'
if ($handler === 'init') {
    $stmt = $db->query("SELECT * FROM tasks ORDER BY created_at ASC");
    $tasks = $stmt->fetchAll(PDO::FETCH_ASSOC);
    
    $html = "";
    foreach ($tasks as $task) {
        $id = htmlspecialchars($task['id'], ENT_QUOTES, 'UTF-8');
        $txt = htmlspecialchars($task['text'], ENT_QUOTES, 'UTF-8');
        $html .= "
            <div class='todo-item' n-id='item_$id'>
                <span class='text'>$txt</span>
                <button class='btn-delete' n-id='btn_$id' n-click='delete:$id'>SUPPRIMER</button>
            </div>";
    }
    if ($html) $p->replaceInner('todo_list', $html);
    $p->send(); exit;
}

if ($handler === 'add_todo') {
    $new_task = trim($formData['todo_input'] ?? '');
    if ($new_task) {
        $id = uniqid();
        $stmt = $db->prepare("INSERT INTO tasks (id, text) VALUES (?, ?)");
        $stmt->execute([$id, $new_task]);
        $safe_id = htmlspecialchars($id, ENT_QUOTES, 'UTF-8');
        $safe_task = htmlspecialchars($new_task, ENT_QUOTES, 'UTF-8');
        $p->appendHtml('todo_list', "
            <div class='todo-item' n-id='item_$safe_id'>
                <span class='text'>$safe_task</span>
                <button class='btn-delete' n-id='btn_$safe_id' n-click='delete:$safe_id'>SUPPRIMER</button>
            </div>
        ")->setText('todo_input', '');
    }
} elseif (strpos($handler, 'delete:') === 0) {
    $target_id = str_replace('delete:', '', $handler);
    $stmt = $db->prepare("DELETE FROM tasks WHERE id = ?");
    $stmt->execute([$target_id]);
    
    // Rafraîchissement total pour robustesse
    $stmt = $db->query("SELECT * FROM tasks ORDER BY created_at ASC");
    $tasks = $stmt->fetchAll(PDO::FETCH_ASSOC);
    $html = "";
    foreach ($tasks as $task) {
        $id = htmlspecialchars($task['id'], ENT_QUOTES, 'UTF-8');
        $txt = htmlspecialchars($task['text'], ENT_QUOTES, 'UTF-8');
        $html .= "
            <div class='todo-item' n-id='item_$id'>
                <span class='text'>$txt</span>
                <button class='btn-delete' n-id='btn_$id' n-click='delete:$id'>SUPPRIMER</button>
            </div>";
    }
    $p->replaceInner('todo_list', $html);
}

$p->send();
