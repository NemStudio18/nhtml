<?php
namespace App\Core;

class Auth {
    public static function check(): bool {
        if (session_status() === PHP_SESSION_NONE) session_start();
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
        session_destroy();
    }

    public static function requireAdmin(): void {
        if (!self::check()) {
            header('Location: /login');
            exit;
        }
    }
}
