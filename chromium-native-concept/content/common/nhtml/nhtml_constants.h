#ifndef CONTENT_COMMON_NHTML_CONSTANTS_H_
#define CONTENT_COMMON_NHTML_CONSTANTS_H_

namespace nhtml {

// MIME type détecté par NhtmlURLLoaderThrottle
// Le serveur DOIT envoyer ce Content-Type pour activer le mode souverain
constexpr char kNhtmlMimeType[]       = "application/x-nhtml-stream";
constexpr char kNhtmlVersionHeader[]  = "X-Nhtml-Version";
constexpr char kNhtmlInterceptHeader[]= "X-Nhtml-Intercept";
constexpr char kNhtmlVersion[]        = "0.1";

// Codes de primitives protocole
constexpr uint8_t kPktHello  = 0x01;
constexpr uint8_t kPktPatch  = 0x02;
constexpr uint8_t kPktEvent  = 0x03;
constexpr uint8_t kPktBind   = 0x04;
constexpr uint8_t kPktSync   = 0x05;
constexpr uint8_t kPktPing   = 0x06;
constexpr uint8_t kPktBTree  = 0x07;
constexpr uint8_t kPktErr    = 0x08;

// Codes d'erreur protocole
constexpr uint8_t kErrUnknownNode     = 0x01;
constexpr uint8_t kErrChecksumFail    = 0x02;
constexpr uint8_t kErrProtoVersion    = 0x03;
constexpr uint8_t kErrPayloadTooLarge = 0x04;
constexpr uint8_t kErrDecompressFail  = 0x05;
constexpr uint8_t kErrUnknownOp      = 0x06;
constexpr uint8_t kErrSessionExpired  = 0x07;
constexpr uint8_t kErrRateLimited     = 0x08;
constexpr uint8_t kErrBindConflict    = 0x09;

// Sévérité des erreurs
constexpr uint8_t kSevWarn  = 0x01;
constexpr uint8_t kSevError = 0x02;
constexpr uint8_t kSevFatal = 0x03;

// Codes op PATCH
constexpr uint8_t kOpSetText      = 0x01;
constexpr uint8_t kOpSetAttr      = 0x02;
constexpr uint8_t kOpDelAttr      = 0x03;
constexpr uint8_t kOpAddClass     = 0x04;
constexpr uint8_t kOpDelClass     = 0x05;
constexpr uint8_t kOpInsertBefore = 0x06;
constexpr uint8_t kOpInsertAfter  = 0x07;
constexpr uint8_t kOpRemove       = 0x08;
constexpr uint8_t kOpSetStyle     = 0x09;
constexpr uint8_t kOpReplaceInner = 0x0A;
constexpr uint8_t kOpScrollTo     = 0x0B;
constexpr uint8_t kOpFocus        = 0x0C;

// Codes event (CLIENT → SERVEUR)
constexpr uint8_t kEvtClick   = 0x01;
constexpr uint8_t kEvtInput   = 0x02;
constexpr uint8_t kEvtSubmit  = 0x03;
constexpr uint8_t kEvtKeydown = 0x04;
constexpr uint8_t kEvtScroll  = 0x05;
constexpr uint8_t kEvtCustom  = 0x06;

// Listen mask bits (NodeSpec.listen_mask)
constexpr uint8_t kListenClick   = 0x01;
constexpr uint8_t kListenInput   = 0x02;
constexpr uint8_t kListenSubmit  = 0x04;
constexpr uint8_t kListenKeydown = 0x08;
constexpr uint8_t kListenScroll  = 0x10;

// Limites
constexpr uint16_t kMaxNodeId       = 0xFFFF;
constexpr size_t   kMaxPayloadSmall = 0xFFFF;     // paquets standards (u16)
constexpr size_t   kMaxPayloadBTree = 0xFFFFFFFF; // B-TREE (u32)
constexpr double   kDefaultFrameBudgetMs = 8.0;
constexpr size_t   kRingBufferInitialSize = 65536; // 64 KB

}  // namespace nhtml

#endif  // CONTENT_COMMON_NHTML_CONSTANTS_H_