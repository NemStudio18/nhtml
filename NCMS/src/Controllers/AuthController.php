<?php
namespace App\Controllers;

use App\Core\Auth;
use App\Core\View;

class AuthController {
    public function showLogin(): void {
        if (Auth::check()) {
            header('Location: /admin');
            exit;
        }
        View::render('login', ['site_name' => 'NCMS Admin']);
    }

    public function login(): void {
        $user = $_POST['user'] ?? '';
        $pass = $_POST['pass'] ?? '';

        if (\App\Core\Auth::login($user, $pass)) {
            \App\Core\Logger::info("Connexion réussie pour l'utilisateur : $user");
            header('Location: /admin');
            exit;
        }

        \App\Core\Logger::warning("Échec de connexion pour l'utilisateur : $user");
        View::render('login', [
            'site_name' => 'NCMS Admin',
            'error_msg' => 'Identifiants invalides'
        ]);
    }

    public function logout(): void {
        Auth::logout();
        header('Location: /');
        exit;
    }
}
