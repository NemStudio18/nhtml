// content/renderer/nhtml/nhtml_local_action_runner.cc
#include "content/renderer/nhtml/nhtml_local_action_runner.h"

#include <cmath>
#include <sstream>
#include <string>

#include "base/logging.h"
#include "base/strings/string_split.h"

// Forward declarations pour Scaffold (remplacés par vrais includes Blink)
namespace blink {
class Element;
class Document;
}

namespace nhtml {

// ─── Constructeur / Destructeur ────────────────────────────────────────────

NhtmlLocalActionRunner::NhtmlLocalActionRunner(blink::Document* document)
    : document_(document) {
  DCHECK(document_);
  LOG(INFO) << "[NHTML] LocalActionRunner initialized.";
}

NhtmlLocalActionRunner::~NhtmlLocalActionRunner() = default;

// ─── RegisterActions ───────────────────────────────────────────────────────
// Point d'entrée principal — câble tous les listeners pour un nœud donné.

void NhtmlLocalActionRunner::RegisterActions(
    uint16_t node_id,
    blink::Element* element,
    const std::vector<LocalAction>& actions) {
  DCHECK(element);

  for (const auto& action : actions) {
    switch (action.trigger) {
      case LocalActionTrigger::kHover:
        AttachHoverListener(element, action);
        break;

      case LocalActionTrigger::kScrollViewport:
        AttachScrollViewportListener(element, action);
        break;

      case LocalActionTrigger::kScrollProgress:
        AttachScrollProgressListener(element, action);
        break;

      case LocalActionTrigger::kMouseMoveWindow:
      case LocalActionTrigger::kMouseMoveSelf:
        AttachMouseMoveListener(element, action);
        break;

      case LocalActionTrigger::kClickLocal:
        AttachClickLocalListener(element, action);
        break;

      case LocalActionTrigger::kFocus:
        // TODO: addEventListener("focus") + addEventListener("blur")
        VLOG(1) << "[NHTML] LocalAction FOCUS registered for node param="
                << action.param;
        break;

      case LocalActionTrigger::kDrag:
        // TODO: HTML5 drag & drop natif Blink
        VLOG(1) << "[NHTML] LocalAction DRAG registered for node param="
                << action.param;
        break;
    }
  }

  VLOG(1) << "[NHTML] RegisterActions: node=" << node_id
          << " count=" << actions.size();
}

void NhtmlLocalActionRunner::UnregisterNode(uint16_t node_id) {
  // TODO: Stocker et détacher les EventListener* par node_id
  // Pour le scaffold, log uniquement
  VLOG(1) << "[NHTML] UnregisterNode: node=" << node_id;
}

// ─── Handlers de déclencheurs ──────────────────────────────────────────────

void NhtmlLocalActionRunner::AttachHoverListener(blink::Element* el,
                                                  const LocalAction& action) {
  // Dans le vrai fork Chromium :
  //
  //   auto* enter_listener = MakeGarbageCollected<NhtmlLocalEventListener>(
  //       weak_factory_.GetWeakPtr(), el, action, /*active=*/true);
  //   el->addEventListener(event_type_names::kMouseenter, enter_listener, false);
  //
  //   if (action.flags.reverse_leave) {
  //     auto* leave_listener = MakeGarbageCollected<NhtmlLocalEventListener>(
  //         weak_factory_.GetWeakPtr(), el, action, /*active=*/false);
  //     el->addEventListener(event_type_names::kMouseleave, leave_listener, false);
  //   }
  //
  // Pour le scaffold, on simule l'effet en loggant :
  VLOG(1) << "[NHTML] HOVER listener attached — action_type=0x"
          << std::hex << static_cast<int>(action.type)
          << " param=\"" << action.param << "\""
          << " reverse=" << action.flags.reverse_leave;
}

void NhtmlLocalActionRunner::AttachScrollViewportListener(
    blink::Element* el, const LocalAction& action) {
  // Dans le vrai fork Chromium, on utiliserait IntersectionObserver natif :
  //
  //   IntersectionObserverInit* init = IntersectionObserverInit::Create();
  //   init->setThreshold({action.threshold_x10 / 10.0});
  //   auto* callback = MakeGarbageCollected<NhtmlIntersectionCallback>(
  //       weak_factory_.GetWeakPtr(), el, action);
  //   IntersectionObserver::Create(init, callback, document_)->observe(el);
  //
  VLOG(1) << "[NHTML] SCROLL_VIEWPORT listener attached — param=\""
          << action.param << "\""
          << " threshold=" << (action.threshold_x10 / 10.0)
          << " once=" << action.flags.once;
}

void NhtmlLocalActionRunner::AttachScrollProgressListener(
    blink::Element* el, const LocalAction& action) {
  // Dans le vrai fork :
  //   document_->addEventListener(event_type_names::kScroll, listener, true);
  //   Le listener calcule :
  //     double progress = scrollY / (scrollHeight - clientHeight);
  //     SetCssVar(el, action.param, std::to_string(progress));
  //
  VLOG(1) << "[NHTML] SCROLL_PROGRESS listener attached — var=\""
          << action.param << "\"";
}

void NhtmlLocalActionRunner::AttachMouseMoveListener(blink::Element* el,
                                                      const LocalAction& action) {
  // Dans le vrai fork :
  //   EventTarget* target = action.flags.scope_self
  //       ? static_cast<EventTarget*>(el)
  //       : static_cast<EventTarget*>(document_->domWindow());
  //
  //   target->addEventListener(event_type_names::kMousemove, listener, false);
  //
  //   Le listener normalise :
  //     double nx = (event.clientX / window.innerWidth)  * 2.0 - 1.0;
  //     double ny = (event.clientY / window.innerHeight) * 2.0 - 1.0;
  //     SetCssVar(root_el, action.param, std::to_string(nx));
  //
  VLOG(1) << "[NHTML] MOUSEMOVE listener attached — var=\""
          << action.param << "\""
          << " scope_self=" << action.flags.scope_self;
}

void NhtmlLocalActionRunner::AttachClickLocalListener(blink::Element* el,
                                                       const LocalAction& action) {
  // Dans le vrai fork :
  //   el->addEventListener(event_type_names::kClick, listener, false);
  //   Le listener appelle ApplyEffect(target_el, action, !current_state)
  //   sans jamais appeler NhtmlSocketWriter.
  //
  VLOG(1) << "[NHTML] CLICK_LOCAL listener attached — param=\""
          << action.param << "\"";
}

// ─── Applicateurs d'effets ─────────────────────────────────────────────────

void NhtmlLocalActionRunner::ApplyEffect(blink::Element* el,
                                          const LocalAction& action,
                                          bool active) {
  switch (action.type) {
    case LocalActionType::kAddClass:
      ToggleClass(el, action.param, active);
      break;

    case LocalActionType::kRemoveClass:
      ToggleClass(el, action.param, !active);
      break;

    case LocalActionType::kToggleClass:
      // TODO: el->classList()->toggle(action.param)
      break;

    case LocalActionType::kSetStyle:
      if (active) {
        ApplyInlineStyle(el, action.param);
      } else {
        // TODO: retirer le style précédemment posé
      }
      break;

    case LocalActionType::kSetCssVarScroll:
    case LocalActionType::kSetCssVarMouseX:
    case LocalActionType::kSetCssVarMouseY:
    case LocalActionType::kSetCssVarMousePx:
      // Appelé directement par les listeners scroll/mouse avec la valeur calculée
      break;

    case LocalActionType::kToggleTarget: {
      // action.param = n-id de la cible
      // TODO: retrouver l'élément cible via NhtmlNodeTable, toggle class "open"
      VLOG(1) << "[NHTML] ToggleTarget: target_nid=\"" << action.param << "\"";
      break;
    }

    case LocalActionType::kDragEnable:
      // TODO: el->setAttribute("draggable", "true")
      break;
  }
}

void NhtmlLocalActionRunner::SetCssVar(blink::Element* el,
                                        const std::string& var_name,
                                        const std::string& value) {
  // Dans le vrai fork :
  //   el->style()->setProperty(document_->GetExecutionContext(),
  //                            AtomicString(var_name),
  //                            AtomicString(value),
  //                            g_empty_atom,
  //                            exception_state);
  VLOG(2) << "[NHTML] SetCssVar " << var_name << " = " << value;
}

void NhtmlLocalActionRunner::ToggleClass(blink::Element* el,
                                          const std::string& class_name,
                                          bool add) {
  // Dans le vrai fork :
  //   if (add) el->classList()->add({AtomicString(class_name)}, exception_state);
  //   else      el->classList()->remove({AtomicString(class_name)}, exception_state);
  VLOG(2) << "[NHTML] ToggleClass " << class_name << " add=" << add;
}

void NhtmlLocalActionRunner::ApplyInlineStyle(blink::Element* el,
                                               const std::string& prop_val) {
  // prop_val format : "transform:scale(1.05)" ou "color:#ff0000"
  auto sep = prop_val.find(':');
  if (sep == std::string::npos) return;

  const std::string prop  = prop_val.substr(0, sep);
  const std::string value = prop_val.substr(sep + 1);

  // Dans le vrai fork :
  //   el->style()->setProperty(ctx, AtomicString(prop), AtomicString(value),
  //                            g_empty_atom, exception_state);
  VLOG(2) << "[NHTML] ApplyInlineStyle " << prop << ": " << value;
}

}  // namespace nhtml
