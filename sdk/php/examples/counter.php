<?php
/**
 * Exemple de compteur avec NHTML SDK v0.2.1
 * 
 * Cet exemple reçoit une action du Gateway Rust (clic sur un bouton)
 * et renvoie un patch de modification de l'interface.
 */

require_once __DIR__ . '/../src/Nhtml.php';

use Nhtml\Nhtml;

// Simulation de récupération d'état (dans un cas réel, cela viendrait d'une session ou DB)
$currentValue = isset($_GET['val']) ? (int)$_GET['val'] : 0;
$nextValue = $currentValue + 1;

// Création du patch de réponse
// 1. On met à jour le texte du compteur
// 2. On change la couleur si on dépasse 10 (pour l'exemple)
$patch = Nhtml::patch()
    ->setText('display-count', "Valeur : $nextValue");

if ($nextValue > 10) {
    $patch->setStyle('display-count', 'color', '#ff4444')
          ->addClass('display-count', 'limit-reached');
}

// Envoi au Gateway Rust
$patch->send();
