<?php
/**
 * NHTML v0.4.0 - Démo Formulaire Live
 */

require_once __DIR__ . '/../../sdk/php/src/Nhtml.php';
require_once __DIR__ . '/../../sdk/php/src/Patch.php';

use Nhtml\Nhtml;

// --- Lecture via STDIN ---
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$formData = json_decode($input['payload'] ?? '{}', true);

$nid = $input['source_id'] ?? '';
// Note: Dans ce formulaire, on se base sur le NID (Email, Password, etc.)
// Le Gateway envoie le NID dans 'handler' pour les inputs simples si configuré,
// ou on peut le retrouver via le mapping. 
// Ici, on va simplement utiliser le 'handler' car bridge.js l'envoie.
$value = trim($formData[$handler] ?? '');
$p = Nhtml::patch();

if ($handler === 'init' || !$handler) {
    $p->setText('hint_email', 'Saisissez votre email')
      ->setText('hint_password', '8 caractères, 1 chiffre requis')
      ->setText('hint_username', 'Lettres, chiffres, underscore uniquement');
    $p->send(); exit;
}

switch ($handler) {
    case 'field_email':
        if ($value === '') {
            $p->setText('hint_email', 'Saisissez votre email')
              ->setAttr('hint_email', 'class', 'field-hint hint-info')
              ->setAttr('field_email', 'class', '');
        } elseif (filter_var($value, FILTER_VALIDATE_EMAIL)) {
            $p->setText('hint_email', '✓ Email valide')
              ->setAttr('hint_email', 'class', 'field-hint hint-ok')
              ->setAttr('field_email', 'class', 'valid');
        } else {
            $p->setText('hint_email', '✗ Format invalide (ex: vous@domaine.com)')
              ->setAttr('hint_email', 'class', 'field-hint hint-err')
              ->setAttr('field_email', 'class', 'invalid');
        }
        break;

    case 'field_password':
        $len = strlen($value);
        $digit = preg_match('/\d/', $value);
        if ($value === '') {
            $p->setText('hint_password', '8 caractères, 1 chiffre requis')
              ->setAttr('hint_password', 'class', 'field-hint hint-info')
              ->setAttr('field_password', 'class', '');
        } elseif ($len >= 8 && $digit) {
            $p->setText('hint_password', '✓ Mot de passe fort')
              ->setAttr('hint_password', 'class', 'field-hint hint-ok')
              ->setAttr('field_password', 'class', 'valid');
        } else {
            $msg = $len < 8 ? "✗ Trop court ({$len}/8 car.)" : '✗ Au moins 1 chiffre requis';
            $p->setText('hint_password', $msg)
              ->setAttr('hint_password', 'class', 'field-hint hint-err')
              ->setAttr('field_password', 'class', 'invalid');
        }
        break;

    case 'field_username':
        if ($value === '') {
            $p->setText('hint_username', 'Lettres, chiffres, underscore uniquement')
              ->setAttr('hint_username', 'class', 'field-hint hint-info')
              ->setAttr('field_username', 'class', '');
        } elseif (preg_match('/^[a-zA-Z0-9_]{3,20}$/', $value)) {
            $p->setText('hint_username', "✓ @{$value} est disponible")
              ->setAttr('hint_username', 'class', 'field-hint hint-ok')
              ->setAttr('field_username', 'class', 'valid');
        } else {
            $p->setText('hint_username', '✗ 3–20 car., lettres/chiffres/_ uniquement')
              ->setAttr('hint_username', 'class', 'field-hint hint-err')
              ->setAttr('field_username', 'class', 'invalid');
        }
        break;
}

// --- Logique d'activation du bouton Submit (Basée sur l'état persistant de TOUS les nœuds) ---
$nodes = $input['nodes'] ?? [];

$email_val = $nodes['field_email']['val'] ?? ($formData['field_email'] ?? '');
$pass_val  = $nodes['field_password']['val'] ?? ($formData['field_password'] ?? '');
$user_val  = $nodes['field_username']['val'] ?? ($formData['field_username'] ?? '');

$email_ok = filter_var($email_val, FILTER_VALIDATE_EMAIL);
$pass_ok  = strlen($pass_val) >= 8 && preg_match('/\d/', $pass_val);
$user_ok  = preg_match('/^[a-zA-Z0-9_]{3,20}$/', $user_val);

if ($email_ok && $pass_ok && $user_ok) {
    $p->delAttr('btn_submit', 'disabled');
} else {
    $p->setAttr('btn_submit', 'disabled', 'disabled');
}

$p->send();
