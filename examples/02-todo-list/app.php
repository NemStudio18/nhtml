<?php
header('Content-Type: application/json');

$db_file = __DIR__ . '/tasks.json';
if (!file_exists($db_file)) {
    file_put_contents($db_file, json_encode([]));
}

$tasks = json_decode(file_get_contents($db_file), true);
$input = json_decode(file_get_contents('php://input'), true);

$patches = [];

if (isset($input['nhtml_event'])) {
    $event = $input['nhtml_event'];
    
    // ACTION : AJOUTER
    if ($event === 'click' && $input['node_id'] === 'btn_add') {
        $new_task = trim($input['form_data']['task_input'] ?? 'Nouvelle tâche');
        if ($new_task !== "") {
            $id = uniqid();
            $tasks[] = ['id' => $id, 'text' => $new_task];
            file_put_contents($db_file, json_encode($tasks));
            
            // On génère l'HTML binaire pour l'item
            $item_html = "<li class='todo-item' id='item_$id'>
                            <span>$new_task</span>
                            <button id='del_$id'>SUPPR</button>
                          </li>";
            
            // Patch 1: Ajouter l'élément à la liste
            $patches[] = [
                'op' => 'append_html',
                'nid' => 'list_container',
                'value' => $item_html
            ];
            
            // Patch 2: Vider l'input
            $patches[] = [
                'op' => 'set_text',
                'nid' => 'task_input',
                'value' => ''
            ];
        }
    }
    
    // ACTION : SUPPRIMER
    if ($event === 'click' && strpos($input['node_id'], 'del_') === 0) {
        $target_id = str_replace('del_', '', $input['node_id']);
        $tasks = array_values(array_filter($tasks, function($t) use ($target_id) {
            return $t['id'] !== $target_id;
        }));
        file_put_contents($db_file, json_encode($tasks));
        
        // Patch: Supprimer le nœud du DOM
        $patches[] = [
            'op' => 'remove_node',
            'nid' => 'item_' . $target_id
        ];
    }
}

// Mise à jour du compteur (systématique)
$patches[] = [
    'op' => 'set_text',
    'nid' => 'task_count',
    'value' => count($tasks)
];

echo json_encode($patches);
