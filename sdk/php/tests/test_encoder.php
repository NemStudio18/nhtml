<?php

require_once __DIR__ . '/../src/Protocol/OpCodes.php';
require_once __DIR__ . '/../src/Protocol/Encoder.php';

use Nhtml\SDK\Protocol\OpCodes;
use Nhtml\SDK\Protocol\Encoder;

echo "🧪 Test de l'encodeur NBPS v0.2.2...\n";

// 1. Test HELLO
// Format: Op(1) + Ver(4) + Len(2) + SID(n)
// 0x01 + 0x00000001 + 0x03 + "sid"
$hello = Encoder::hello(1, "sid");
$expectedHello = bin2hex(pack('C', 1) . pack('N', 1) . pack('C', 3) . "sid");
$actualHello = bin2hex($hello);

if ($actualHello === $expectedHello) {
    echo "✅ HELLO : OK ($actualHello)\n";
} else {
    echo "❌ HELLO : Erreur !\n   Attendu : $expectedHello\n   Obtenu   : $actualHello\n";
}

// 2. Test PATCH (SetText)
// Format: Op(1) + Count(2) + [Target(2) + OpType(1) + Ver(4) + ValLen(2) + Val(n)]
$ops = [
    [
        'target_id' => 42,
        'type' => OpCodes::OP_SET_TEXT,
        'version' => 5,
        'value' => "Hello"
    ]
];
$patch = Encoder::patch($ops);
$actualPatch = bin2hex($patch);
$expectedPatch = "030001002a0100000005000548656c6c6f"; // Calculé manuellement

if ($actualPatch === strtolower($expectedPatch)) {
    echo "✅ PATCH (SetText) : OK ($actualPatch)\n";
} else {
    echo "❌ PATCH (SetText) : Erreur !\n   Attendu : $expectedPatch\n   Obtenu   : $actualPatch\n";
}
