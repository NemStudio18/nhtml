<?php
/**
 * NHTML Patch v0.4.0
 * Représente une collection de mutations DOM à envoyer au client.
 */

namespace Nhtml;

class Patch {
    protected array $ops = [];
    protected array $joinRooms = [];
    protected array $leaveRooms = [];
    protected ?array $broadcastInstr = null;

    public static function create(): self {
        return new static();
    }

    public function setText(string $nid, string $text): self {
        $this->ops[] = ['op' => 'set_text', 'nid' => $nid, 'val' => $text];
        return $this;
    }

    public function addClass(string $nid, string $class): self {
        $this->ops[] = ['op' => 'add_class', 'nid' => $nid, 'val' => $class];
        return $this;
    }

    public function removeClass(string $nid, string $class): self {
        $this->ops[] = ['op' => 'del_class', 'nid' => $nid, 'val' => $class];
        return $this;
    }

    public function setStyle(string $nid, string $prop, string $val): self {
        $this->ops[] = ['op' => 'set_style', 'nid' => $nid, 'prop' => $prop, 'val' => $val];
        return $this;
    }

    public function setAttr(string $nid, string $key, string $val): self {
        $this->ops[] = ['op' => 'set_attr', 'nid' => $nid, 'key' => $key, 'val' => $val];
        return $this;
    }

    public function delAttr(string $nid, string $key): self {
        $this->ops[] = ['op' => 'del_attr', 'nid' => $nid, 'key' => $key];
        return $this;
    }

    public function replaceInner(string $nid, string $html): self {
        $this->ops[] = ['op' => 'replace_inner', 'nid' => $nid, 'val' => $html];
        return $this;
    }

    public function appendHtml(string $nid, string $html): self {
        $this->ops[] = ['op' => 'append_html', 'nid' => $nid, 'val' => $html];
        return $this;
    }

    public function remove(string $nid): self {
        $this->ops[] = ['op' => 'remove', 'nid' => $nid];
        return $this;
    }

    public function focus(string $nid): self {
        $this->ops[] = ['op' => 'focus', 'nid' => $nid];
        return $this;
    }

    public function scrollTo(string $nid): self {
        $this->ops[] = ['op' => 'scroll_to', 'nid' => $nid];
        return $this;
    }

    /**
     * Marque la dernière opération comme devant être diffusée à tous les clients
     * @deprecated Utilisez broadcastInRoom() ou broadcastToOthers() pour plus de contrôle
     */
    public function broadcast(bool $b = true): self {
        if (!empty($this->ops)) {
            $this->ops[count($this->ops) - 1]['broadcast'] = $b;
        }
        return $this;
    }

    /**
     * Configure une diffusion groupée explicite (v0.6.0)
     */
    public function broadcastToAll(array $ops): self {
        $this->broadcastInstr = ['scope' => 'all', 'patch' => $ops];
        return $this;
    }

    public function broadcastToOthers(array $ops): self {
        $this->broadcastInstr = ['scope' => 'others', 'patch' => $ops];
        return $this;
    }

    public function broadcastInRoom(string $roomId, array $ops): self {
        $this->broadcastInstr = ['scope' => 'room', 'room_id' => $roomId, 'patch' => $ops];
        return $this;
    }

    public function broadcastToSession(string $sessionId, array $ops): self {
        $this->broadcastInstr = ['scope' => 'direct', 'target_sid' => $sessionId, 'patch' => $ops];
        return $this;
    }

    /**
     * Gestion des Salons
     */
    public function joinRoom(string $roomId): self {
        $this->joinRooms[] = $roomId;
        return $this;
    }

    public function leaveRoom(string $roomId): self {
        $this->leaveRooms[] = $roomId;
        return $this;
    }

    public function getOps(): array {
        return $this->ops;
    }

    /**
     * Envoie la réponse au Gateway Rust
     */
    public function send(): void {
        if (PHP_SAPI !== 'cli' && !headers_sent()) {
            header('Content-Type: application/json');
        }

        $response = [
            'patch' => $this->ops
        ];

        if (!empty($this->joinRooms)) {
            $response['join_room'] = $this->joinRooms;
        }

        if (!empty($this->leaveRooms)) {
            $response['leave_room'] = $this->leaveRooms;
        }

        if ($this->broadcastInstr) {
            $response['broadcast'] = $this->broadcastInstr;
        }

        echo json_encode($response);
        exit;
    }
}
