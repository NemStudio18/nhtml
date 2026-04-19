<?php
namespace App\Core;

class Auth {
    public static function check(): bool {
        if (session_status() === PHP_SESSION_NONE) session_start();
        
        // Mode dev/debug bypass
        $config = json_decode(file_get_contents(__DIR__ . '/../../config.json'), true);
        if (isset($config['debug']) && $config['debug'] === true) {
            return true;
        }

        return isset($_SESSION['admin_logged']) && $_SESSION['admin_logged'] === true;
    }

    public static function login(string $user, string $pass): bool {
        $config = json_decode(file_get_contents(__DIR__ . '/../../config.json'), true);
        
        if ($user === $config['admin_user'] && $pass === $config['admin_pass']) {
            if (session_status() === PHP_SESSION_NONE) session_start();
            $_SESSION['admin_logged'] = true;
            return true;
        }
        return false;
    }

    public static function logout(): void {
        if (session_status() === PHP_SESSION_NONE) session_start();
        $_SESSION = [];
        if (ini_get("session.use_cookies")) {
            $params = session_get_cookie_params();
            setcookie(session_name(), '', time() - 42000,
                $params["path"], $params["domain"],
                $params["secure"], $params["httponly"]
            );
        }
        session_destroy();
    }

    public static function requireAdmin(): void {
        if (!self::check()) {
            header('Location: /login');
            exit;
        }
    }
}
