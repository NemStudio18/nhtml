<?php
// php-sdk/src/Patch.php
// Surface développeur pour créer des mutations DOM.
// Le développeur ne manipule jamais les node_id binaires.
// Il utilise les n-id métier définis dans ses fichiers .nhtml.

namespace Nhtml;

/**
 * Une opération de mutation DOM.
 * Sérialisée en JSON, transmise au Gateway Rust qui la convertit en binaire.
 */
final class PatchOp
{
    private function __construct(
        public readonly string $op,
        public readonly ?string $nid     = null,
        public readonly ?int    $node_id = null,
        public readonly ?string $value   = null,
        public readonly ?string $key     = null,
        public readonly ?string $prop    = null,
    ) {}

    public function toArray(): array
    {
        return array_filter([
            'op'      => $this->op,
            'nid'     => $this->nid,
            'node_id' => $this->node_id,
            'value'   => $this->value,
            'key'     => $this->key,
            'prop'    => $this->prop,
        ], fn($v) => $v !== null);
    }

    // ── Factory methods ────────────────────────────────────────────────────

    public static function setText(string $nid, string $text): self
    {
        return new self('set_text', nid: $nid, value: $text);
    }

    public static function addClass(string $nid, string $class): self
    {
        return new self('add_class', nid: $nid, value: $class);
    }

    public static function removeClass(string $nid, string $class): self
    {
        return new self('del_class', nid: $nid, value: $class);
    }

    public static function setAttr(string $nid, string $attr, string $val): self
    {
        return new self('set_attr', nid: $nid, key: $attr, value: $val);
    }

    public static function setStyle(string $nid, string $prop, string $val): self
    {
        return new self('set_style', nid: $nid, prop: $prop, value: $val);
    }

    public static function show(string $nid): self
    {
        return new self('del_attr', nid: $nid, key: 'hidden');
    }

    public static function hide(string $nid): self
    {
        return new self('set_attr', nid: $nid, key: 'hidden', value: 'true');
    }

    public static function remove(string $nid): self
    {
        return new self('remove', nid: $nid);
    }

    public static function focus(string $nid): self
    {
        return new self('focus', nid: $nid);
    }

    public static function replaceInner(string $nid, string $html): self
    {
        return new self('replace_inner', nid: $nid, value: $html);
    }
}

/**
 * Facade statique — la surface que le développeur utilise dans ses controllers.
 *
 * Exemple :
 *   return [
 *       Patch::setText('compteur', '5'),
 *       Patch::addClass('btn', 'active'),
 *   ];
 */
final class Patch
{
    private function __construct() {}

    public static function setText(string $nid, string $text): PatchOp
    {
        return PatchOp::setText($nid, $text);
    }

    public static function addClass(string $nid, string $class): PatchOp
    {
        return PatchOp::addClass($nid, $class);
    }

    public static function removeClass(string $nid, string $class): PatchOp
    {
        return PatchOp::removeClass($nid, $class);
    }

    public static function setAttr(string $nid, string $attr, string $val): PatchOp
    {
        return PatchOp::setAttr($nid, $attr, $val);
    }

    public static function setStyle(string $nid, string $prop, string $val): PatchOp
    {
        return PatchOp::setStyle($nid, $prop, $val);
    }

    public static function show(string $nid): PatchOp
    {
        return PatchOp::show($nid);
    }

    public static function hide(string $nid): PatchOp
    {
        return PatchOp::hide($nid);
    }

    public static function remove(string $nid): PatchOp
    {
        return PatchOp::remove($nid);
    }

    public static function focus(string $nid): PatchOp
    {
        return PatchOp::focus($nid);
    }

    public static function replaceInner(string $nid, string $html): PatchOp
    {
        return PatchOp::replaceInner($nid, $html);
    }
}
