<?php
/**
 * NHTML SDK v0.2.3 - Backend avec BDD Métier Autonome
 * La vraie persistance de l'application est gérée ici.
 */

header('Content-Type: application/json');

// --- 1. Initialisation de la BDD Métier (SQLite) ---
$dbPath = __DIR__ . '/counter.db';
$db = new PDO('sqlite:' . $dbPath);
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Création de la table métier si elle n'existe pas
$db->exec("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY, counter_value INTEGER DEFAULT 0)");

// Initialisation de la première ligne si la base est vide
$stmt = $db->query("SELECT counter_value FROM state WHERE id = 1");
$row = $stmt->fetch(PDO::FETCH_ASSOC);
if (!$row) {
    $db->exec("INSERT INTO state (id, counter_value) VALUES (1, 0)");
    $counter = 0;
} else {
    $counter = (int)$row['counter_value'];
}
// --- Fin Initialisation ---

$input = json_decode(file_get_contents('php://input'), true);
$event = $input['nhtml_event'] ?? '';

$patches = [];

if ($event === 'init') {
    // Synchronisation initiale avec la valeur métier
    $patches[] = [
        'op'    => 'set_text',
        'nid'   => 'counter_value',
        'value' => (string)$counter
    ];
} 
elseif ($event === 'click') {
    $nodeId = $input['node_id'] ?? '';
    
    if ($nodeId === 'btn_increment') {
        // Incrémentation dans la BDD Métier
        $counter++;
        $updateStmt = $db->prepare("UPDATE state SET counter_value = :val WHERE id = 1");
        $updateStmt->execute([':val' => $counter]);
        
        // On renvoie le patch UI au Gateway
        $patches[] = [
            'op'    => 'log',
            'value' => "Incrémentation réussie : nouveau compteur = $counter"
        ];

        $patches[] = [
            'op'    => 'set_text',
            'nid'   => 'counter_value',
            'value' => (string)$counter
        ];
    }
}

echo json_encode($patches);
