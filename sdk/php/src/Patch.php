<?php
/**
 * NHTML Patch v0.4.0
 * Représente une collection de mutations DOM à envoyer au client.
 */

namespace Nhtml;

class Patch {
    protected array $ops = [];

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
     */
    public function broadcast(bool $b = true): self {
        if (!empty($this->ops)) {
            $this->ops[count($this->ops) - 1]['broadcast'] = $b;
        }
        return $this;
    }

    public function getOps(): array {
        return $this->ops;
    }

    /**
     * Envoie la réponse au Gateway Rust (Format tableau brut attendu par socket/mod.rs)
     */
    public function send(): void {
        if (PHP_SAPI !== 'cli' && !headers_sent()) {
            header('Content-Type: application/json');
        }
        echo json_encode($this->ops);
        exit;
    }
}
