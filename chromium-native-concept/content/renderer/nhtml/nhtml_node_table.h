#ifndef CONTENT_RENDERER_NHTML_NHTML_NODE_TABLE_H_
#define CONTENT_RENDERER_NHTML_NHTML_NODE_TABLE_H_

#include <array>
#include <optional>
#include <vector>

#include "content/common/nhtml/nhtml_types.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/dom/element.h"
#include "third_party/blink/renderer/platform/heap/persistent.h"

namespace nhtml {

class NhtmlEventListener;
class NhtmlSocketWriter;

// ─── NhtmlNodeEntry ────────────────────────────────────────────────────────
//
// Une entrée dans la Node Table.
// Le Persistent<> épingle l'Element dans le GC Oilpan de Blink :
// il ne sera collecté que quand Unbind() libère le handle.

struct NhtmlNodeEntry {
  // Handle fort vers l'objet DOM — survit aux GC passes
  blink::Persistent<blink::Node> element;

  // Métadonnées de l'arbre (miroir en mémoire WASM de la spec)
  uint16_t parent_id    = 0;
  uint16_t first_child  = 0;  // 0 = feuille
  uint16_t next_sibling = 0;  // 0 = dernier de sa fratrie

  uint8_t listen_mask = 0;

  // Listeners natifs attachés (pour DetachListeners propre)
  std::vector<blink::Persistent<NhtmlEventListener>> listeners;

  // Accumulateur de styles inline (flush groupé par frame)
  // Evite N appels setProperty() → un seul recalcul de style
  std::vector<std::pair<std::string, std::string>> pending_styles;
};

// ─── NhtmlNodeTable ────────────────────────────────────────────────────────
//
// Table plate indexée par node_id (uint16).
// Accès O(1) garanti pour toutes les opérations PATCH critiques.
//
// Thread safety :
//   Toutes les méthodes DOIVENT être appelées sur le main thread Blink.
//   Le NhtmlInterpreter (IO thread) ne touche jamais directement cette table.

class NhtmlNodeTable {
 public:
  explicit NhtmlNodeTable(blink::Document* document,
                          NhtmlSocketWriter* socket_writer);
  ~NhtmlNodeTable() = default;

  // ── Construction depuis B-TREE ──────────────────────────────────────────

  // Point d'entrée principal : construit l'arbre DOM complet
  // depuis la ParsedTree produite par l'IO thread.
  // Doit être appelé sur le main thread.
  void BuildFromTree(ParsedTree tree);

  // ── Opérations PATCH ────────────────────────────────────────────────────

  void ApplyPatchOp(const PatchOp& op);

  // Flush groupé des styles accumulés (appelé une fois par batch)
  void FlushPendingStyles();

  // ── Accès ───────────────────────────────────────────────────────────────

  // Résolution O(1) : node_id → blink::Node*
  // Retourne nullptr si l'ID est inconnu
  blink::Node*    ResolveNode(uint16_t id);
  blink::Element* Resolve(uint16_t id);

  size_t bound_count() const;

 private:
  // ── Construction interne (appelée depuis BuildFromTree) ─────────────────

  void CreateAndBindNode(const NodeSpec& spec);
  void AttachToParent(const NodeSpec& spec);

  // ── Gestion des listeners natifs ────────────────────────────────────────

  void AttachListeners(uint16_t id);
  void DetachListeners(uint16_t id);

  // ── Libération récursive ────────────────────────────────────────────────

  void Unbind(uint16_t id);
  void UnbindSubtree(uint16_t root_id);

  // ── Membres ─────────────────────────────────────────────────────────────

  // Tableau plat : 65 536 slots, accès O(1) par index direct
  std::array<std::optional<NhtmlNodeEntry>, 65536> entries_;

  // Pool d'IDs libérés, réutilisables pour les INSERT dynamiques
  std::vector<uint16_t> free_ids_;

  uint16_t root_id_ = 0;

  // Références non-owning (lifetimes garanties par le Document)
  blink::Document*   document_;       // owned par le renderer
  NhtmlSocketWriter* socket_writer_;  // owned par NhtmlInterpreter
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_NODE_TABLE_H_