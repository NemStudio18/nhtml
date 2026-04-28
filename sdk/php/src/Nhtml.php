<?php
/**
 * NHTML SDK PHP v0.4.0
 * Point d'entrée principal pour la logique métier NHTML.
 */

namespace Nhtml;

class Nhtml {
    /**
     * Crée un nouveau patch de modifications.
     * @return Patch
     */
    public static function patch(): Patch {
        return new Patch();
    }

    /**
     * Démarre un groupe d'opérations (Batch)
     */
    public static function batch(callable $callback): void {
        $patch = new Patch();
        $callback($patch);
        $patch->send();
    }

    /**
     * Rejoint un salon (Room) immédiatement.
     */
    public static function joinRoom(string $roomId): void {
        (new Patch())->joinRoom($roomId)->send();
    }

    /**
     * Quitte un salon (Room) immédiatement.
     */
    public static function leaveRoom(string $roomId): void {
        (new Patch())->leaveRoom($roomId)->send();
    }
}
