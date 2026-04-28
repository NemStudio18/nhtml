<?php
/**
 * NHTML v0.4.0 - Routeur Industriel
 * Redirige les requêtes du Gateway vers les fichiers app.php locaux.
 */

$uri = $_SERVER['REQUEST_URI'];
$path = parse_url($uri, PHP_URL_PATH);

// Sécurité : Interdire l'accès aux fichiers sensibles
if (strpos($path, '..') !== false) {
    http_response_code(403);
    exit("Forbidden");
}

// Si on demande un fichier qui existe (image, css), on le sert
if (file_exists(__DIR__ . $path) && !is_dir(__DIR__ . $path) && !str_ends_with($path, '.php')) {
    return false;
}

// Recherche du app.php dans le dossier demandé
// Exemple : /examples/counter/ -> /examples/counter/app.php
$app_php = __DIR__ . $path;
if (is_dir($app_php)) {
    $app_php = rtrim($app_php, '/') . '/app.php';
}

if (file_exists($app_php)) {
    include $app_php;
} else {
    http_response_code(404);
    echo json_encode(["error" => "App not found at $app_php"]);
}
