#include "content/renderer/nhtml/nhtml_node_table.h"

#include "base/check.h"
#include "base/logging.h"
#include "content/common/nhtml/nhtml_constants.h"
#include "content/renderer/nhtml/nhtml_event_listener.h"
#include "content/renderer/nhtml/nhtml_socket_writer.h"
#include "third_party/blink/renderer/core/css/css_style_declaration.h"
#include "third_party/blink/renderer/core/dom/element.h"
#include "third_party/blink/renderer/core/dom/text.h"
#include "third_party/blink/renderer/core/html/forms/html_form_element.h"
#include "third_party/blink/renderer/core/html/forms/html_input_element.h"
#include "third_party/blink/renderer/core/style/style_change_reason.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"

namespace nhtml {

NhtmlNodeTable::NhtmlNodeTable(blink::Document* document,
                               NhtmlSocketWriter* socket_writer)
    : document_(document), socket_writer_(socket_writer) {
  DCHECK(document_);
  DCHECK(socket_writer_);
  entries_.fill(std::nullopt);
}

// ─── BuildFromTree ─────────────────────────────────────────────────────────
//
// Trois phases séparées intentionnellement :
//   1. Créer tous les éléments DOM (sans les connecter)
//   2. Construire la hiérarchie parent → enfant (sans reflows intermédiaires)
//   3. Attacher la racine au body (un seul layout pass)

void NhtmlNodeTable::BuildFromTree(ParsedTree tree) {
  DCHECK(IsMainThread());

  if (tree.has_error()) {
    LOG(ERROR) << "[Nhtml] B-TREE build error, code="
               << static_cast<int>(tree.error_code());
    return;
  }

  // Phase 1 : créer et binder tous les nœuds
  // L'ordre depth-first dans tree.nodes est garanti par le parser IO thread
  for (const NodeSpec& spec : tree.nodes) {
    CreateAndBindNode(spec);
  }

  // Phase 2 : construire la hiérarchie
  // Séparé de la création → zéro reflow intermédiaire
  for (const NodeSpec& spec : tree.nodes) {
    AttachToParent(spec);
  }

  // Phase 3 : insérer la racine dans le document
  blink::Element* root = Resolve(tree.root_id);
  if (!root) {
    LOG(ERROR) << "[Nhtml] Root node " << tree.root_id << " introuvable";
    return;
  }

  root_id_ = tree.root_id;
  document_->body()->AppendChild(root, ASSERT_NO_EXCEPTION);

  // Une seule invalidation globale — acceptable uniquement au premier affichage
  document_->GetStyleEngine().MarkAllElementsForStyleRecalc(
      StyleChangeReasonForTracing::Create("nhtml-init"));

  LOG(INFO) << "[Nhtml] BuildFromTree OK — "
            << tree.nodes.size() << " nœuds liés";
}

// ─── CreateAndBindNode ─────────────────────────────────────────────────────

void NhtmlNodeTable::CreateAndBindNode(const NodeSpec& spec) {
  DCHECK(IsMainThread());

  if (spec.node_type == 0x02) {
    // TEXT NODE — pas d'attributs, pas de listeners
    blink::Text* text = document_->createTextNode(
        WTFString::FromUTF8(spec.text.c_str()));

    entries_[spec.id] = NhtmlNodeEntry{
        .element      = blink::Persistent<blink::Node>(text),
        .parent_id    = spec.parent_id,
        .first_child  = 0,
        .next_sibling = 0,
        .listen_mask  = 0,
    };
    return;
  }

  // ELEMENT NODE
  // CreateElement() = même factory que le parser HTML, sans tokenisation
  blink::QualifiedName qname(
      g_null_atom,
      AtomicString(WTFString::FromUTF8(spec.tag.c_str())),
      blink::html_names::xhtmlNamespaceURI);

  blink::Element* el = document_->CreateElement(
      qname, blink::CreateElementFlags::ByCreateElement());

  // Injecter les attributs
  // Note : les Mutation Observers ne sont pas encore actifs sur ces nœuds
  // (non connectés au document), donc pas de callbacks intempestifs
  for (const auto& [key, val] : spec.attrs) {
    el->setAttribute(
        QualifiedName(AtomicString(WTFString::FromUTF8(key.c_str()))),
        AtomicString(WTFString::FromUTF8(val.c_str())),
        ASSERT_NO_EXCEPTION);
  }

  entries_[spec.id] = NhtmlNodeEntry{
      .element      = blink::Persistent<blink::Element>(el),
      .parent_id    = spec.parent_id,
      .first_child  = 0,
      .next_sibling = 0,
      .listen_mask  = spec.listen_mask,
  };

  // Attacher les listeners natifs immédiatement
  if (spec.listen_mask != 0) {
    AttachListeners(spec.id);
  }
}

// ─── AttachToParent ────────────────────────────────────────────────────────

void NhtmlNodeTable::AttachToParent(const NodeSpec& spec) {
  if (spec.parent_id == 0) return;  // racine

  blink::Node* child  = ResolveNode(spec.id);
  blink::Node* parent = ResolveNode(spec.parent_id);

  if (!child || !parent) {
    LOG(WARNING) << "[Nhtml] AttachToParent: nœud introuvable id="
                 << spec.id << " parent=" << spec.parent_id;
    return;
  }

  parent->AppendChild(child, ASSERT_NO_EXCEPTION);

  // Mettre à jour la linked list dans la Node Table
  auto& parent_entry = entries_[spec.parent_id];
  if (parent_entry->first_child == 0) {
    parent_entry->first_child = spec.id;
  } else {
    // Parcourir jusqu'au dernier sibling
    uint16_t sibling = parent_entry->first_child;
    while (entries_[sibling].has_value() &&
           entries_[sibling]->next_sibling != 0) {
      sibling = entries_[sibling]->next_sibling;
    }
    if (entries_[sibling].has_value()) {
      entries_[sibling]->next_sibling = spec.id;
    }
  }
}

// ─── ApplyPatchOp ──────────────────────────────────────────────────────────

void NhtmlNodeTable::ApplyPatchOp(const PatchOp& op) {
  DCHECK(IsMainThread());

  blink::Element* el = Resolve(op.target_id);
  if (!el) {
    LOG(WARNING) << "[Nhtml] PATCH sur nœud inconnu id=" << op.target_id;
    // Émettre ERR 0x01 UNKNOWN_NODE via socket (non-fatal)
    return;
  }

  switch (op.type) {

    case kOpSetText: {
      // Modifier le Text node enfant direct si possible
      // Evite la destruction/recréation du sous-arbre
      if (auto* text = blink::DynamicTo<blink::Text>(el->firstChild())) {
        text->setData(WTFString::FromUTF8(op.value.c_str()));
      } else {
        el->setTextContent(WTFString::FromUTF8(op.value.c_str()));
      }
      break;
    }

    case kOpSetAttr: {
      el->setAttribute(
          QualifiedName(AtomicString(WTFString::FromUTF8(op.attr_name.c_str()))),
          AtomicString(WTFString::FromUTF8(op.value.c_str())),
          ASSERT_NO_EXCEPTION);
      break;
    }

    case kOpDelAttr: {
      el->removeAttribute(
          QualifiedName(AtomicString(WTFString::FromUTF8(op.attr_name.c_str()))));
      break;
    }

    case kOpAddClass: {
      el->classList().Add(
          AtomicString(WTFString::FromUTF8(op.value.c_str())),
          ASSERT_NO_EXCEPTION);
      break;
    }

    case kOpDelClass: {
      el->classList().Remove(
          AtomicString(WTFString::FromUTF8(op.value.c_str())));
      break;
    }

    case kOpSetStyle: {
      // Accumuler — flush groupé dans FlushPendingStyles()
      entries_[op.target_id]->pending_styles.emplace_back(
          op.prop, op.value);
      break;
    }

    case kOpRemove: {
      el->remove();
      Unbind(op.target_id);
      break;
    }

    case kOpReplaceInner: {
      // Fallback HTMX-like : uniquement si une op chirurgicale
      // n'est pas possible (ex: contenu généré dynamiquement côté serveur)
      el->setInnerHTML(
          WTFString::FromUTF8(op.value.c_str()),
          ASSERT_NO_EXCEPTION);
      break;
    }

    case kOpScrollTo: {
      // Lecture nécessaire — schedulée en phase "reads" par le dispatcher
      el->scrollIntoView();
      break;
    }

    case kOpFocus: {
      el->focus();
      break;
    }

    default:
      LOG(WARNING) << "[Nhtml] op inconnue: 0x"
                   << std::hex << static_cast<int>(op.type);
  }

  // Invalidation chirurgicale : seulement ce nœud
  el->SetNeedsStyleRecalc(
      blink::kLocalStyleChange,
      StyleChangeReasonForTracing::Create("nhtml-patch"));
}

// ─── FlushPendingStyles ────────────────────────────────────────────────────

void NhtmlNodeTable::FlushPendingStyles() {
  DCHECK(IsMainThread());

  for (auto& entry : entries_) {
    if (!entry.has_value()) continue;
    if (entry->pending_styles.empty()) continue;

    blink::Element* el =
        blink::DynamicTo<blink::Element>(entry->element.Get());
    if (!el) continue;

    blink::CSSStyleDeclaration* style = el->style();
    for (const auto& [prop, val] : entry->pending_styles) {
      style->setProperty(
          WTFString::FromUTF8(prop.c_str()),
          WTFString::FromUTF8(val.c_str()),
          /*priority=*/ g_empty_atom,
          ASSERT_NO_EXCEPTION);
    }
    entry->pending_styles.clear();
  }
}

// ─── Accès ─────────────────────────────────────────────────────────────────

blink::Node* NhtmlNodeTable::ResolveNode(uint16_t id) {
  const auto& slot = entries_[id];
  if (!slot.has_value()) return nullptr;
  return slot->element.Get();
}

blink::Element* NhtmlNodeTable::Resolve(uint16_t id) {
  return blink::DynamicTo<blink::Element>(ResolveNode(id));
}

size_t NhtmlNodeTable::bound_count() const {
  size_t count = 0;
  for (const auto& e : entries_) {
    if (e.has_value()) ++count;
  }
  return count;
}

// ─── Listeners natifs ──────────────────────────────────────────────────────

void NhtmlNodeTable::AttachListeners(uint16_t id) {
  auto& entry = entries_[id];
  DCHECK(entry.has_value());

  blink::Element* el   = Resolve(id);
  uint8_t         mask = entry->listen_mask;
  if (!el) return;

  struct ListenerSpec {
    uint8_t     mask_bit;
    const char* event_name;
    uint8_t     proto_code;
  };

  static const ListenerSpec specs[] = {
      { kListenClick,   "click",   kEvtClick   },
      { kListenInput,   "input",   kEvtInput   },
      { kListenSubmit,  "submit",  kEvtSubmit  },
      { kListenKeydown, "keydown", kEvtKeydown },
      { kListenScroll,  "scroll",  kEvtScroll  },
  };

  for (const auto& spec : specs) {
    if (!(mask & spec.mask_bit)) continue;

    auto* listener = blink::MakeGarbageCollected<NhtmlEventListener>(
        id, spec.proto_code, socket_writer_);

    el->addEventListener(
        AtomicString(spec.event_name),
        listener,
        /*use_capture=*/false);

    entry->listeners.push_back(
        blink::Persistent<NhtmlEventListener>(listener));
  }
}

void NhtmlNodeTable::DetachListeners(uint16_t id) {
  auto& entry = entries_[id];
  if (!entry.has_value()) return;

  blink::Element* el = Resolve(id);
  if (!el) return;

  for (auto& listener : entry->listeners) {
    el->removeEventListener(
        AtomicString(listener->EventName()),
        listener.Get(),
        /*use_capture=*/false);
  }
  entry->listeners.clear();
}

// ─── Libération ────────────────────────────────────────────────────────────

void NhtmlNodeTable::Unbind(uint16_t id) {
  if (!entries_[id].has_value()) return;

  DetachListeners(id);
  UnbindSubtree(entries_[id]->first_child);
  entries_[id].reset();           // libère le Persistent → Oilpan peut GC
  free_ids_.push_back(id);        // recycler l'ID
}

void NhtmlNodeTable::UnbindSubtree(uint16_t root_id) {
  if (root_id == 0) return;
  if (!entries_[root_id].has_value()) return;

  uint16_t next = entries_[root_id]->next_sibling;
  UnbindSubtree(entries_[root_id]->first_child);
  entries_[root_id].reset();
  free_ids_.push_back(root_id);
  UnbindSubtree(next);
}

}  // namespace nhtml