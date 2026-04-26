<?php
/**
 * NHTML v0.4.0 - Style Lab
 */

require_once __DIR__ . '/../../sdk/php/src/Nhtml.php';
require_once __DIR__ . '/../../sdk/php/src/Patch.php';

use Nhtml\Nhtml;

// --- Lecture via STDIN ---
$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$formData = json_decode($input['payload'] ?? '{}', true);

$nid = $input['source_id'] ?? $handler;

$value = $formData[$nid] ?? '';
$p = Nhtml::patch();

if ($handler === 'init' || !$handler) {
    // Initialiser les labels
    $p->setText('lbl_radius', "12px")->setText('lbl_scale', "1.00×")->setText('lbl_shadow', "40%");
    $p->send(); exit;
}

$colors = [
    'color_0' => '#38bdf8', 'color_1' => '#a78bfa', 'color_2' => '#34d399',
    'color_3' => '#f472b6', 'color_4' => '#fb923c', 'color_5' => '#facc15',
];

if ($handler === 'slider_radius') {
    $p->setText('lbl_radius', "{$value}px")->setStyle('preview_box', 'border-radius', "{$value}px");
} elseif ($handler === 'slider_scale') {
    $scaleVal = floatval($value) / 100;
    $scaleStr = sprintf("%.2f", $scaleVal);
    $p->setText('lbl_scale', "{$scaleStr}×")->setStyle('preview_box', 'transform', "scale({$scaleVal})");
} elseif ($handler === 'slider_shadow') {
    $shadowVal = floatval($value) / 100;
    $p->setText('lbl_shadow', "{$value}%")->setStyle('preview_box', 'box-shadow', "0 20px 60px rgba(0,0,0,{$shadowVal})");
} elseif ($handler === 'select') {
    $target_nid = $input['source_id'] ?? '';
    $color = $colors[$target_nid] ?? '#38bdf8';
    foreach ($colors as $id => $_) {
        $p->setAttr($id, 'class', ($id === $target_nid) ? 'color-swatch active' : 'color-swatch');
    }
    $p->setStyle('preview_box', 'background', $color);
}

$p->setText('stat_last', $nid)->send();
