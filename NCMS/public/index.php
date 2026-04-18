<?php
// Autoloader simple pour PSR-4
spl_autoload_register(function ($class) {
    $prefix = 'App\\';
    $base_dir = __DIR__ . '/../src/';

    $len = strlen($prefix);
    if (strncmp($prefix, $class, $len) !== 0) {
        return;
    }

    $relative_class = substr($class, $len);
    $file = $base_dir . str_replace('\\', '/', $relative_class) . '.php';

    if (file_exists($file)) {
        require $file;
    }
});

use App\Core\Logger;

// Gestionnaire d'erreurs globales
set_exception_handler(function ($e) {
    Logger::error("Exception non gérée : " . $e->getMessage() . " in " . $e->getFile() . " on line " . $e->getLine());
    
    if ($e instanceof \App\Core\NotFoundException) {
        http_response_code(404);
        $errorPage = __DIR__ . '/error/404.html';
        if (file_exists($errorPage)) { readfile($errorPage); } else { echo "404 - Page non trouvée"; }
    } else {
        http_response_code(500);
        $errorPage = __DIR__ . '/error/500.html';
        if (file_exists($errorPage)) { readfile($errorPage); } else { echo "Erreur interne."; }
    }
    exit;
});

set_error_handler(function ($errno, $errstr, $errfile, $errline) {
    if (!(error_reporting() & $errno)) return false;
    Logger::warning("PHP Error ($errno): $errstr in $errfile on line $errline");
    return false;
});

use App\Core\Router;
use App\Controllers\BlogController;
use App\Controllers\AuthController;
use App\Controllers\AdminController;

$router = new Router();

// Routes Front
$router->add('GET', '/', [BlogController::class, 'index']);
$router->add('GET', '/post/{id}', [BlogController::class, 'show']);
$router->add('GET', '/page/{id}', [BlogController::class, 'showPage']);
$router->add('GET', '/category/{id}', [BlogController::class, 'category']);
$router->add('POST', '/form/submit', [BlogController::class, 'submitForm']);

// Routes Auth
$router->add('GET', '/login', [AuthController::class, 'showLogin']);
$router->add('POST', '/login', [AuthController::class, 'login']);
$router->add('GET', '/logout', [AuthController::class, 'logout']);

// Routes Admin
$router->add('GET', '/admin', [AdminController::class, 'dashboard']);
$router->add('GET', '/admin/create', [AdminController::class, 'create']);
$router->add('GET', '/admin/edit/{id}', [AdminController::class, 'edit']);
$router->add('POST', '/admin/save', [AdminController::class, 'save']);
$router->add('POST', '/admin/delete', [AdminController::class, 'delete']);
$router->add('POST', '/admin/category/save', [AdminController::class, 'saveCategory']);
$router->add('POST', '/admin/category/delete', [AdminController::class, 'deleteCategory']);
$router->add('POST', '/admin/menu/save', [AdminController::class, 'saveMenu']);
$router->add('POST', '/admin/menu/delete', [AdminController::class, 'deleteMenu']);
$router->add('POST', '/admin/form/save', [AdminController::class, 'saveForm']);
$router->add('POST', '/admin/form/delete', [AdminController::class, 'deleteForm']);

$router->dispatch();

// Fallback 404 (si le routeur ne gère pas la 404 lui-même)
