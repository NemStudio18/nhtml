<?php
/**
 * NHTML SDK PHP v0.2.1
 * Cœur du système de réponse pour le Gateway Rust.
 */

namespace Nhtml;

class Nhtml {
    private array $ops = [];
    private bool $isBatch = false;

    /**
     * Initialise un nouveau patch de modifications
     */
    public static function patch(): self {
        return new self();
    }

    /**
     * Démarre un groupe d'opérations (Batch)
     */
    public static function batch(callable $callback): void {
        $instance = new self();
        $instance->isBatch = true;
        $callback($instance);
        $instance->send();
    }

    /**
     * Change le texte d'un élément
     */
    public function setText(string $nid, string $text): self {
        $this->ops[] = [
            'op' => 'setText',
            'nid' => $nid,
            'val' => $text
        ];
        return $this;
    }

    /**
     * Ajoute une classe CSS
     */
    public function addClass(string $nid, string $class): self {
        $this->ops[] = [
            'op' => 'addClass',
            'nid' => $nid,
            'val' => $class
        ];
        return $this;
    }

    /**
     * Retire une classe CSS
     */
    public function removeClass(string $nid, string $class): self {
        $this->ops[] = [
            'op' => 'removeClass',
            'nid' => $nid,
            'val' => $class
        ];
        return $this;
    }

    /**
     * Modifie un style CSS en ligne
     */
    public function setStyle(string $nid, string $prop, string $val): self {
        $this->ops[] = [
            'op' => 'setStyle',
            'nid' => $nid,
            'prop' => $prop,
            'val' => $val
        ];
        return $this;
    }

    /**
     * Remplace le contenu HTML interne (équivalent innerHTML)
     */
    public function replaceInner(string $nid, string $html): self {
        $this->ops[] = [
            'op' => 'replaceInner',
            'nid' => $nid,
            'val' => $html
        ];
        return $this;
    }

    /**
     * Envoie la réponse au Gateway Rust
     */
    public function send(): void {
        if (!headers_sent()) {
            header('Content-Type: application/json');
        }
        echo json_encode([
            'protocol' => 'nhtml/0.2.1',
            'status' => 'success',
            'patch' => $this->ops
        ]);
    }
}
