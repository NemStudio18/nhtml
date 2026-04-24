// router.php - Injecte les headers CORS et gère le Fallback HTTP (Mode Mutualisé)
header("Access-Control-Allow-Origin: *");
header("Access-Control-Allow-Methods: GET, POST, OPTIONS");
header("Access-Control-Allow-Headers: Content-Type, X-NHTML-Session");

if ($_SERVER["REQUEST_METHOD"] === "OPTIONS") exit(0);

// --- GESTION DU FALLBACK BINAIRE (Mode Mutualisé) ---
if ($_SERVER["REQUEST_METHOD"] === "POST" && $_SERVER["CONTENT_TYPE"] === "application/octet-stream") {
    $rawInput = file_get_contents("php://input");
    if (strlen($rawInput) >= 5) {
        $type = ord($rawInput[0]);
        // [0x02][NodeID:4]
        if ($type === 0x02) {
            $nodeId = unpack("N", substr($rawInput, 1, 4))[1];
            
            // Simulation de l'appel Gateway -> App
            $ch = curl_init("http://127.0.0.1:8000/counter/app.php");
            curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode([
                "nhtml_event" => "click",
                "node_id" => ($nodeId === 1) ? "btn_increment" : $nodeId
            ]));
            curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
            $jsonResponse = curl_exec($ch);
            curl_close($ch);
            
            $patches = json_decode($jsonResponse, true);
            
            // Encodage manuel en binaire PATCH (v0.3.1)
            // [0x03][Len:4][Count:2] ...
            $payload = "";
            $count = 0;
            foreach ($patches as $p) {
                if ($p['op'] === 'set_text') {
                    $targetId = ($p['nid'] === 'counter_value') ? 2 : (int)$p['nid'];
                    $val = $p['value'];
                    $payload .= pack("n", $targetId); // u16
                    $payload .= pack("C", 0x01);      // OP_SET_TEXT
                    $payload .= pack("N", 1);         // Version fixée à 1 pour POC
                    $payload .= pack("n", strlen($val));
                    $payload .= $val;
                    $count++;
                }
            }
            
            header("Content-Type: application/octet-stream");
            echo pack("C", 0x03); // Type
            echo pack("N", strlen($payload) + 2); // Len
            echo pack("n", $count); // Count
            echo $payload;
            exit;
        }
    }
}

$path = $_SERVER["DOCUMENT_ROOT"] . parse_url($_SERVER["REQUEST_URI"], PHP_URL_PATH);
if (file_exists($path) && !is_dir($path)) {
    $ext = pathinfo($path, PATHINFO_EXTENSION);
    if ($ext === 'js' || $ext === 'mjs') header('Content-Type: application/javascript');
    elseif ($ext === 'wasm') header('Content-Type: application/wasm');
    elseif ($ext === 'css') header('Content-Type: text/css');
    elseif ($ext === 'html' || $ext === 'nhtml') header('Content-Type: text/html');
    
    if ($ext !== 'php') {
        readfile($path);
        return true;
    }
}

return false;
