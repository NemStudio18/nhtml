<?php
/**
 * NHTML v0.4.0 - Démo Compteur (Industrial Version)
 */

require_once __DIR__ . '/../../sdk/php/src/Nhtml.php';
require_once __DIR__ . '/../../sdk/php/src/Patch.php';

use Nhtml\Nhtml;

// --- Lecture du contexte via STDIN (Stateless Industrial Bridge) ---
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$nodes = $input['nodes'] ?? [];

// Récupérer la valeur actuelle depuis l'état de session maintenu par le Gateway
$counter = (int)($nodes['counter_value']['val'] ?? 0);

if ($handler === 'increment') {
    $counter++;
    Nhtml::patch()->setText('counter_value', (string)$counter)->send();
    exit;
}

// Fallback init / No match
if ($handler === 'init' || !$handler) {
    Nhtml::patch()->setText('counter_value', (string)$counter)->send();
} else {
    Nhtml::patch()->send();
}
