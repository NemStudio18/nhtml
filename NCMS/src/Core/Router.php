<?php
namespace App\Core;

class Router {
    private array $routes = [];

    public function add(string $method, string $path, callable|array $callback): void {
        $path = preg_replace('/\//', '\\/', $path);
        $path = preg_replace('/\{([a-z]+)\}/', '(?P<\1>[^\/]+)', $path);
        $this->routes[] = [
            'method' => $method,
            'path' => '/^' . $path . '$/',
            'callback' => $callback
        ];
    }

    public function dispatch(): void {
        $method = $_SERVER['REQUEST_METHOD'];
        $uri = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
        
        // Retirer le sous-répertoire éventuel (si lancé via php -S localhost...)
        $baseUrl = str_replace('/index.php', '', $_SERVER['SCRIPT_NAME']);
        $uri = str_replace($baseUrl, '', $uri);
        if (empty($uri)) $uri = '/';

        foreach ($this->routes as $route) {
            if ($route['method'] === $method && preg_match($route['path'], $uri, $matches)) {
                $params = array_filter($matches, 'is_string', ARRAY_FILTER_USE_KEY);
                
                if (is_array($route['callback'])) {
                    [$controller, $action] = $route['callback'];
                    $instance = new $controller();
                    $instance->$action($params);
                } else {
                    call_user_func($route['callback'], $params);
                }
                return;
            }
        }

        http_response_code(404);
        $errorPage = dirname(__DIR__) . '/public/error/404.html';
        if (file_exists($errorPage)) {
            readfile($errorPage);
        } else {
            echo "404 - Page non trouvée";
        }
    }
}
