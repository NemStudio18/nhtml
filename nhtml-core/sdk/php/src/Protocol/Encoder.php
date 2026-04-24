<?php

namespace Nhtml\SDK\Protocol;

/**
 * NBPS v0.2.2 Binary Encoder
 */
class Encoder {
    /**
     * Encode un message HELLO
     */
    public static function hello(int $version, string $sessionId): string {
        $bin = pack('C', OpCodes::HELLO);
        $bin .= pack('N', $version);
        $bin .= pack('C', strlen($sessionId));
        $bin .= $sessionId;
        return $bin;
    }

    /**
     * Encode un message PING
     */
    public static function ping(int $sequence): string {
        $bin = pack('C', OpCodes::PING);
        $bin .= pack('C', $sequence);
        return $bin;
    }

    /**
     * Encode un message PATCH complet
     * @param array $ops Liste d'opérations [['target_id' => 42, 'type' => OpCodes::OP_SET_TEXT, 'version' => 5, 'value' => '...'], ...]
     */
    public static function patch(array $ops): string {
        $bin = pack('C', OpCodes::PATCH);
        $bin .= pack('n', count($ops));

        foreach ($ops as $op) {
            $bin .= pack('n', $op['target_id']);
            $bin .= pack('C', $op['type']);
            $bin .= pack('N', $op['version']);

            // Encodage de la valeur selon l'opération
            switch ($op['type']) {
                case OpCodes::OP_SET_TEXT:
                case OpCodes::OP_REPLACE_INNER:
                case OpCodes::OP_ADD_CLASS:
                case OpCodes::OP_DEL_CLASS:
                    $val = (string)$op['value'];
                    $bin .= pack('n', strlen($val));
                    $bin .= $val;
                    break;
                
                // TODO: Autres types d'opérations
            }
        }

        return $bin;
    }
}
