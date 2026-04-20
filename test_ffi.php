<?php
require_once __DIR__ . '/nhtml_engine/NhtmlCompiler.php';

use Nhtml\NhtmlCompiler;

$source = '
<var name="test" value="42">
<var name="hero" value=\'{"name": "Zelda", "hp": 100}\'>
<div class="test">
    <h1>Hello {test}</h1>
    <button on:click="test++">Inc</button>
    <empty for="[1]">Vide</empty>
</div>';

echo "Compilation via NhtmlCompiler...\n";
$start = microtime(true);
$res = NhtmlCompiler::compile($source);
$end = microtime(true);

echo "Mode utilisé : " . $res['mode'] . "\n";
echo "Temps d'exécution : " . round(($end - $start) * 1000, 2) . " ms\n";
echo "HTML Généré : \n" . trim($res['html']) . "\n";
echo "Manifest JSON : \n" . json_encode($res['manifest'], JSON_PRETTY_PRINT) . "\n";
