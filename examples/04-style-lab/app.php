<?php
header('Content-Type: application/json');
$input = json_decode(file_get_contents('php://input'), true);

if (!isset($input['nhtml_event'])) { echo json_encode([]); exit; }

$nid   = $input['node_id'] ?? '';
$value = $input['form_data'][$nid] ?? '';
$state = $input['current_state'] ?? [];
$patches = [];

// Récupère l'état courant ou des défauts
$radius = intval($state['radius'] ?? 12);
$scale  = intval($state['scale']  ?? 100);
$shadow = intval($state['shadow'] ?? 40);
$color  = $state['color'] ?? '#38bdf8';
$pkts   = intval($state['pkts']   ?? 0) + 1;

$colors = [
    'color_0' => '#38bdf8',
    'color_1' => '#a78bfa',
    'color_2' => '#34d399',
    'color_3' => '#f472b6',
    'color_4' => '#fb923c',
    'color_5' => '#facc15',
];

// Met à jour la valeur selon le contrôle touché
if ($nid === 'slider_radius') {
    $radius = intval($value);
    $patches[] = ['op' => 'set_text', 'nid' => 'lbl_radius', 'value' => "{$radius}px"];
}
elseif ($nid === 'slider_scale') {
    $scale = intval($value);
    $scale_f = number_format($scale / 100, 1);
    $patches[] = ['op' => 'set_text', 'nid' => 'lbl_scale', 'value' => "{$scale_f}×"];
}
elseif ($nid === 'slider_shadow') {
    $shadow = intval($value);
    $patches[] = ['op' => 'set_text', 'nid' => 'lbl_shadow', 'value' => "{$shadow}%"];
}
elseif (isset($colors[$nid])) {
    $color = $colors[$nid];
    // Désactive tous les swatches, active le sélectionné
    foreach ($colors as $id => $_) {
        $cls = ($id === $nid) ? 'color-swatch active' : 'color-swatch';
        $patches[] = ['op' => 'set_class', 'nid' => $id, 'value' => $cls];
    }
}

// Construit le style CSS final
$scale_f  = number_format($scale / 100, 2);
$shadow_a = number_format($shadow / 100, 2);
$rgb      = ltrim($color, '#');
$r = hexdec(substr($rgb, 0, 2));
$g = hexdec(substr($rgb, 2, 2));
$b = hexdec(substr($rgb, 4, 2));

$new_style = implode('; ', [
    "border-radius: {$radius}px",
    "background: {$color}",
    "transform: scale({$scale_f})",
    "box-shadow: 0 20px 60px rgba({$r},{$g},{$b},{$shadow_a})",
    "transition: border-radius 0.1s, background 0.15s, box-shadow 0.15s, transform 0.15s",
    "width: 240px",
    "height: 240px",
    "display: flex",
    "align-items: center",
    "justify-content: center",
    "font-size: 2.5rem",
]);

$patches[] = ['op' => 'set_attr', 'nid' => 'preview_box', 'attr' => 'style', 'value' => $new_style];

// Stats live
$patches[] = ['op' => 'set_text', 'nid' => 'stat_pkts', 'value' => (string)$pkts];
$patches[] = ['op' => 'set_text', 'nid' => 'stat_last',  'value' => $nid];

echo json_encode($patches);
