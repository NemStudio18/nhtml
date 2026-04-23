<?php

namespace Nhtml\SDK;

use Nhtml\SDK\Protocol\Encoder;
use Nhtml\SDK\Protocol\OpCodes;

/**
 * NHTML Gateway SDK v0.2.2 (Professional)
 */
class Gateway {
    private array $pendingOps = [];
    private string $sessionId = '';
    private int $protocolVersion = 1;

    public function __construct(string $sessionId = '') {
        $this->sessionId = $sessionId;
    }

    /**
     * Définit le texte d'un nœud
     */
    public function setText(int $nodeId, string $text, int $version = 0): self {
        $this->pendingOps[] = [
            'target_id' => $nodeId,
            'type' => OpCodes::OP_SET_TEXT,
            'version' => $version,
            'value' => $text
        ];
        return $this;
    }

    /**
     * Ajoute une classe CSS à un nœud
     */
    public function addClass(int $nodeId, string $className, int $version = 0): self {
        $this->pendingOps[] = [
            'target_id' => $nodeId,
            'type' => OpCodes::OP_ADD_CLASS,
            'version' => $version,
            'value' => $className
        ];
        return $this;
    }

    /**
     * Envoie toutes les opérations en attente sous forme de PATCH binaire
     */
    public function send(): void {
        if (empty($this->pendingOps)) {
            return;
        }

        $binary = Encoder::patch($this->pendingOps);
        $this->pendingOps = [];

        $this->output($binary);
    }

    /**
     * Gère l'envoi effectif vers le client (via HTTP standard)
     */
    private function output(string $binary): void {
        if (!headers_sent()) {
            header('Content-Type: application/octet-stream');
            header('X-Nhtml-Protocol: 0.2.2');
        }
        echo $binary;
    }

    /**
     * helper statique pour démarrer un patch rapidement
     */
    public static function patch(): self {
        return new self();
    }
}
