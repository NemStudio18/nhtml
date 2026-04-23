<?php

namespace Nhtml\SDK;

/**
 * Factory pour le SDK NHTML
 */
class GatewayFactory {
    /**
     * Crée une instance de Gateway prête à l'emploi
     */
    public static function create(string $sessionId = ''): Gateway {
        // Dans le futur, on pourrait charger une config ici
        return new Gateway($sessionId);
    }

    /**
     * Raccourci pour un patch rapide
     */
    public static function patch(string $sessionId = ''): Gateway {
        return self::create($sessionId);
    }
}
