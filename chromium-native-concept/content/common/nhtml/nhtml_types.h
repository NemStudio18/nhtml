#ifndef CONTENT_COMMON_NHTML_NHTML_TYPES_H_
#define CONTENT_COMMON_NHTML_NHTML_TYPES_H_

#include <cstdint>
#include <string>
#include <utility>
#include <vector>

namespace nhtml {

// ─── Structs POD (Plain Old Data) ─────────────────────────────────────────
// Ces structs ne contiennent AUCUN objet Blink.
// Ils sont construits sur l'IO thread, puis postés au main thread.

// ─── Local Actions — spec v0.2.1 ──────────────────────────────────────────
// Actions exécutées ENTIÈREMENT côté client (C++ Blink) sans round-trip réseau.
// Encodées en extension du paquet BIND (0x04), champ local_action_count.

// Types d'effets (correspond à la table §5 de NHTML_LOCAL_ACTIONS_v0.1.md)
enum class LocalActionType : uint8_t {
  kAddClass        = 0x01,  // param = nom de classe
  kRemoveClass     = 0x02,  // param = nom de classe
  kToggleClass     = 0x03,  // param = nom de classe
  kSetStyle        = 0x04,  // param = "propriete:valeur"
  kSetCssVarScroll = 0x05,  // param = "--nom-var" (0.0→1.0)
  kSetCssVarMouseX = 0x06,  // param = "--nom-var" (normalisé -1→1)
  kSetCssVarMouseY = 0x07,  // param = "--nom-var" (normalisé -1→1)
  kSetCssVarMousePx= 0x08,  // param = "--nom-var" (pixels absolus)
  kToggleTarget    = 0x09,  // param = n-id de la cible
  kDragEnable      = 0x0A,  // param = groupe de drag
};

// Déclencheurs (correspond à la table §6 de NHTML_LOCAL_ACTIONS_v0.1.md)
enum class LocalActionTrigger : uint8_t {
  kHover          = 0x01,  // mouseenter / mouseleave
  kScrollViewport = 0x02,  // IntersectionObserver (entrée dans viewport)
  kScrollProgress = 0x03,  // position scroll globale (0→1)
  kMouseMoveWindow= 0x04,  // mousemove relatif à la fenêtre
  kMouseMoveSelf  = 0x05,  // mousemove relatif à l'élément
  kFocus          = 0x06,  // focus / blur
  kClickLocal     = 0x07,  // click sans EVENT serveur
  kDrag           = 0x08,  // dragstart / dragend
};

// Flags de comportement pour une Local Action
struct LocalActionFlags {
  bool once          : 1;  // déclencher une seule fois
  bool reverse_leave : 1;  // annuler l'effet au mouse-leave (hover)
  bool scope_self    : 1;  // coordonnées relatives à l'élément (mouse)
  uint8_t reserved   : 5;
};

// Une Local Action unique, issue du champ local_action_count du BIND
struct LocalAction {
  LocalActionType    type      = LocalActionType::kAddClass;
  LocalActionTrigger trigger   = LocalActionTrigger::kHover;
  std::string        param;       // classe, CSS var, "prop:val", n-id cible…
  LocalActionFlags   flags     = {};
  uint8_t            threshold_x10 = 0;  // seuil scroll × 10 (15 = 0.15)
};

struct AttrPair {
  std::string key;
  std::string value;
};

// Représentation d'un nœud issu du parsing du paquet 0x07 (B-TREE)
// Construit sur IO thread, consommé sur main thread.
// Les champs n-* ne sont PAS dans le B-TREE — ils arrivent via BIND (0x04).
struct NodeSpec {
  uint16_t              id              = 0;
  uint16_t              parent_id       = 0;  // 0 = racine
  uint8_t               node_type       = 0;  // 0x01=element, 0x02=text
  std::string           tag;
  std::vector<AttrPair> attrs;
  std::string           text;
  uint8_t               listen_mask     = 0;
  uint16_t              insertion_index = 0;  // ordre depth-first
};

// ─── Paquet BIND (0x04) — spec v0.2 ──────────────────────────────────────
// Transmis par le Gateway après le B-TREE, un par attribut n- détecté.
// Permet d'associer les métadonnées n- à un node_id binaire existant.
struct BindEntry {
  uint16_t    target_id      = 0;
  std::string n_id;           // n-id métier (ex: "compteur")
  std::string selector;       // ex: "[n-id=compteur]" (debug seulement)
  uint8_t     listen_mask    = 0;  // bits: 0=click, 1=input, 2=submit, 3=keydown, 4=focus/blur
  uint8_t     behavior_flags = 0;  // bits: 0=n-live, 1=n-prevent, 2=n-once
  uint8_t     debounce_100ms = 0;  // valeur × 100ms, 0 = désactivé
  std::string handler;        // ex: "panier.ajouter"
  std::string n_model;        // ex: "panier.coupon" (binding bidirectionnel)
  std::string n_text;         // ex: "page.clicks" (contenu contrôlé par PHP)

  // ── Extension v0.2.1 ──────────────────────────────────────────────────────
  std::vector<LocalAction> local_actions;  // effets client-side (zéro réseau)
};

// Arbre parsé, prêt à être consommé par BuildFromTree()
struct ParsedTree {
  std::vector<NodeSpec> nodes;
  uint16_t              root_id     = 0;
  bool                  has_error_  = false;
  uint8_t               error_code_ = 0;

  static ParsedTree Error(uint8_t code) {
    ParsedTree t;
    t.has_error_  = true;
    t.error_code_ = code;
    return t;
  }

  bool has_error() const { return has_error_; }
  uint8_t error_code() const { return error_code_; }
};

// ─── Opérations PATCH ─────────────────────────────────────────────────────

struct PatchOp {
  uint8_t     type      = 0;
  uint16_t    target_id = 0;
  std::string value;      // SET_TEXT, ADD_CLASS, DEL_CLASS, ReplaceInner…
  std::string attr_name;  // SET_ATTR, DEL_ATTR
  std::string prop;       // SET_STYLE : propriété CSS
  uint16_t    ref_id    = 0;  // INSERT_BEFORE / INSERT_AFTER

  // Priorité de scheduling (utilisée par PatchScheduler)
  enum class Priority { High, Normal, Low };
  Priority priority() const;

  // True si l'op nécessite une lecture DOM avant écriture
  // (SCROLL_TO, FOCUS → getBoundingClientRect)
  bool needs_read() const;
};

// ─── Header du paquet B-TREE (0x07) ───────────────────────────────────────

struct BTreeHeader {
  uint8_t  compression    = 0;  // 0x01 = zstd
  uint32_t length_raw     = 0;
  uint32_t checksum       = 0;  // CRC32 du payload décompressé
  uint32_t length_compressed = 0;
};

// ─── Paquet brut sorti du StreamAssembler ─────────────────────────────────

struct NhtmlPacket {
  uint8_t              type    = 0;
  std::vector<uint8_t> payload;
};

// ─── Constantes des types de paquets (spec v0.2) ───────────────────────────

constexpr uint8_t kPktHello  = 0x01;
constexpr uint8_t kPktPatch  = 0x02;
constexpr uint8_t kPktEvent  = 0x03;
constexpr uint8_t kPktBind   = 0x04;
constexpr uint8_t kPktSync   = 0x05;
constexpr uint8_t kPktPing   = 0x06;
constexpr uint8_t kPktBTree  = 0x07;
constexpr uint8_t kPktErr    = 0x08;

// ─── Constantes des codes d'erreur (spec v0.2) ────────────────────────────

constexpr uint8_t kErrUnknownNode     = 0x01;
constexpr uint8_t kErrChecksumFail    = 0x02;
constexpr uint8_t kErrProtoVersion    = 0x03;
constexpr uint8_t kErrPayloadTooLarge = 0x04;
constexpr uint8_t kErrDecompressFail  = 0x05;
constexpr uint8_t kErrUnknownOp       = 0x06;
constexpr uint8_t kErrSessionExpired  = 0x07;
constexpr uint8_t kErrRateLimited     = 0x08;
constexpr uint8_t kErrBindConflict    = 0x09;

}  // namespace nhtml

#endif  // CONTENT_COMMON_NHTML_NHTML_TYPES_H_