<?php

namespace Nhtml\SDK\Protocol;

/**
 * NBPS v0.2.2 OpCodes & Constants
 * Source de vérité pour l'encodage binaire du protocole Native-HTML.
 */
class OpCodes {
    // Message Types (Opcodes principaux)
    public const HELLO  = 0x01;
    public const EVENT  = 0x02;
    public const PATCH  = 0x03;
    public const SYNC   = 0x04;
    public const BTREE  = 0x05; // Full sync
    public const PING   = 0x06;
    public const ERROR  = 0x7F;

    // Patch Operations
    public const OP_SET_TEXT      = 0x01;
    public const OP_SET_ATTR      = 0x02;
    public const OP_DEL_ATTR      = 0x03;
    public const OP_ADD_CLASS     = 0x04;
    public const OP_DEL_CLASS     = 0x05;
    public const OP_INSERT_BEFORE = 0x06;
    public const OP_INSERT_AFTER  = 0x07;
    public const OP_REMOVE        = 0x08;
    public const OP_SET_STYLE     = 0x09;
    public const OP_REPLACE_INNER = 0x0A;
    
    // Severity (for Error packets)
    public const SEV_INFO  = 0x00;
    public const SEV_WARN  = 0x01;
    public const SEV_ERROR = 0x02;
    public const SEV_FATAL = 0x03;
}
