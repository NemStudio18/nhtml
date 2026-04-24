<?php
header('Content-Type: application/json');
$input = json_decode(file_get_contents('php://input'), true);
$patches = [];

if (!isset($input['nhtml_event'])) { echo json_encode([]); exit; }

$nid   = $input['node_id'] ?? '';
$value = trim($input['form_data'][$nid] ?? '');

switch ($nid) {
    case 'field_email':
        if ($value === '') {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_email', 'value'=>'Saisissez votre email'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_email','value'=>''];
        } elseif (filter_var($value, FILTER_VALIDATE_EMAIL)) {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_email', 'value'=>'✓ Email valide'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_email', 'value'=>'field-hint hint-ok'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_email','value'=>'valid'];
        } else {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_email', 'value'=>'✗ Format invalide (ex: vous@domaine.com)'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_email', 'value'=>'field-hint hint-err'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_email','value'=>'invalid'];
        }
        break;

    case 'field_password':
        $len   = strlen($value);
        $digit = preg_match('/\d/', $value);
        if ($value === '') {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_password', 'value'=>'8 caractères, 1 chiffre requis'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_password', 'value'=>'field-hint hint-info'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_password','value'=>''];
        } elseif ($len >= 8 && $digit) {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_password', 'value'=>'✓ Mot de passe fort'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_password', 'value'=>'field-hint hint-ok'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_password','value'=>'valid'];
        } else {
            $msg = $len < 8 ? "✗ Trop court ({$len}/8 car.)" : '✗ Au moins 1 chiffre requis';
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_password', 'value'=>$msg];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_password', 'value'=>'field-hint hint-err'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_password','value'=>'invalid'];
        }
        break;

    case 'field_username':
        if ($value === '') {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_username', 'value'=>'Lettres, chiffres, underscore uniquement'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_username', 'value'=>'field-hint hint-info'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_username','value'=>''];
        } elseif (preg_match('/^[a-zA-Z0-9_]{3,20}$/', $value)) {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_username', 'value'=>"✓ @{$value} est disponible"];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_username', 'value'=>'field-hint hint-ok'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_username','value'=>'valid'];
        } else {
            $patches[] = ['op'=>'set_text',  'nid'=>'hint_username', 'value'=>'✗ 3–20 car., lettres/chiffres/_ uniquement'];
            $patches[] = ['op'=>'set_class', 'nid'=>'hint_username', 'value'=>'field-hint hint-err'];
            $patches[] = ['op'=>'set_class', 'nid'=>'field_username','value'=>'invalid'];
        }
        break;
}

echo json_encode($patches);
