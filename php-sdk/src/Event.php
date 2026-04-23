<?php
// php-sdk/src/Event.php + Dispatcher.php

namespace Nhtml;

/**
 * Représente un EVENT reçu du navigateur, désérialisé depuis le JSON
 * transmis par le Gateway Rust via stdin.
 */
final class NhtmlEvent
{
    public function __construct(
        public readonly string $handler,     // "panier.ajouter"
        public readonly int    $sourceId,    // node_id binaire
        public readonly int    $eventType,   // code protocole
        public readonly array  $payload,     // données de l'event
        public readonly array  $nidMap,      // n-id → node_id
    ) {}

    /**
     * Désérialise depuis le JSON stdin envoyé par le Gateway.
     */
    public static function fromStdin(): self
    {
        $raw = file_get_contents('php://stdin');
        if (!$raw) {
            throw new \RuntimeException('Nhtml: stdin vide');
        }

        $ctx = json_decode($raw, true);
        if (!$ctx) {
            throw new \RuntimeException('Nhtml: JSON stdin invalide');
        }

        // Le payload peut être une string JSON (form data) ou vide
        $payload = [];
        if (!empty($ctx['payload'])) {
            $decoded = json_decode($ctx['payload'], true);
            $payload = is_array($decoded) ? $decoded : ['raw' => $ctx['payload']];
        }

        return new self(
            handler   : $ctx['handler']    ?? '',
            sourceId  : (int)($ctx['source_id']  ?? 0),
            eventType : (int)($ctx['event_type'] ?? 0),
            payload   : $payload,
            nidMap    : $ctx['nid_map']    ?? [],
        );
    }

    /**
     * Résoudre un n-id métier en node_id binaire
     * (utile si le controller a besoin du node_id direct)
     */
    public function resolveNid(string $nid): ?int
    {
        return isset($this->nidMap[$nid]) ? (int)$this->nidMap[$nid] : null;
    }
}

/**
 * Dispatcher : route le handler "module.action" vers la bonne méthode PHP.
 *
 * Format du handler :
 *   "action"           → $this->action($event)
 *   "module.action"    → $controllers['module']->action($event)
 *   "module.action:p"  → $controllers['module']->action($event, "p")
 */
final class EventDispatcher
{
    /** @var array<string, object> */
    private array $controllers = [];

    public function register(string $name, object $controller): void
    {
        $this->controllers[$name] = $controller;
    }

    /**
     * Dispatcher en mode "script simple" :
     * Le script PHP est appelé par le Gateway, lit stdin, dispatch, écrit stdout.
     *
     * Retourne le JSON des PatchOp au Gateway.
     */
    public function run(): void
    {
        $event   = NhtmlEvent::fromStdin();
        $patches = $this->dispatch($event);
        echo $this->serializePatches($patches);
    }

    /**
     * @return PatchOp[]
     */
    public function dispatch(NhtmlEvent $event): array
    {
        if (empty($event->handler)) {
            return [];
        }

        // Parser "module.action:param" ou "action"
        $parts  = explode('.', $event->handler, 2);
        $param  = null;

        if (count($parts) === 2) {
            [$module, $action_raw] = $parts;
            [$action, $param] = array_pad(explode(':', $action_raw, 2), 2, null);
        } else {
            $module = null;
            [$action, $param] = array_pad(explode(':', $parts[0], 2), 2, null);
        }

        // Trouver le controller
        if ($module !== null) {
            if (!isset($this->controllers[$module])) {
                throw new \RuntimeException("Controller '$module' non enregistré");
            }
            $controller = $this->controllers[$module];
        } elseif (count($this->controllers) === 1) {
            $controller = reset($this->controllers);
        } else {
            throw new \RuntimeException(
                "Handler '$event->handler' ambigu — précisez le module"
            );
        }

        if (!method_exists($controller, $action)) {
            throw new \RuntimeException(
                "Méthode '$action' introuvable sur " . get_class($controller)
            );
        }

        // Appel avec ou sans param statique
        $result = $param !== null
            ? $controller->$action($event, $param)
            : $controller->$action($event);

        // Normaliser : le controller peut retourner un tableau ou un seul PatchOp
        if ($result instanceof PatchOp) {
            return [$result];
        }

        return is_array($result) ? $result : [];
    }

    private function serializePatches(array $patches): string
    {
        $ops = array_map(fn(PatchOp $p) => $p->toArray(), $patches);
        return json_encode($ops, JSON_UNESCAPED_UNICODE);
    }
}
