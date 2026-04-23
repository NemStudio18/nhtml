// content/renderer/nhtml/nhtml_local_action_runner.h
#ifndef CONTENT_RENDERER_NHTML_NHTML_LOCAL_ACTION_RUNNER_H_
#define CONTENT_RENDERER_NHTML_NHTML_LOCAL_ACTION_RUNNER_H_

#include <cstdint>
#include <unordered_map>
#include <vector>

#include "base/memory/scoped_refptr.h"
#include "base/memory/weak_ptr.h"
#include "base/task/single_thread_task_runner.h"
#include "content/common/nhtml/nhtml_types.h"

namespace blink {
class Element;
class Document;
}

namespace nhtml {

// ─── NhtmlLocalActionRunner ───────────────────────────────────────────────
//
// Exécute les Local Actions (hover, scroll, mousemove, toggle…) entièrement
// côté client sans aucun round-trip réseau.
//
// Thread model : Main Thread UNIQUEMENT (manipule le DOM Blink)
// Ownership    : NhtmlNodeTable owns NhtmlLocalActionRunner
//
// Fonctionnement :
//   1. RegisterActions() reçoit les LocalAction[] du BindEntry
//   2. Il câble les listeners DOM natifs C++ (addEventListener)
//   3. Chaque listener applique l'effet localement (CSS class, CSS var, style)
//      sans jamais appeler NhtmlSocketWriter (zéro réseau)

class NhtmlLocalActionRunner {
 public:
  explicit NhtmlLocalActionRunner(blink::Document* document);
  ~NhtmlLocalActionRunner();

  // Appelé depuis le Main Thread après réception d'un BIND v0.2.1
  // Câble les listeners pour le nœud donné.
  void RegisterActions(uint16_t node_id,
                       blink::Element* element,
                       const std::vector<LocalAction>& actions);

  // Appelé quand un nœud est supprimé du DOM (REMOVE patch op)
  // Détache tous les listeners associés.
  void UnregisterNode(uint16_t node_id);

 private:
  // ── Handlers de déclencheurs ──────────────────────────────────────────────

  // Câble un listener mouseenter / mouseleave pour un effet HOVER
  void AttachHoverListener(blink::Element* el,
                           const LocalAction& action);

  // Utilise IntersectionObserver natif C++ pour SCROLL_VIEWPORT
  void AttachScrollViewportListener(blink::Element* el,
                                    const LocalAction& action);

  // Abonne à l'event scroll global pour SCROLL_PROGRESS
  void AttachScrollProgressListener(blink::Element* el,
                                    const LocalAction& action);

  // Abonne à mousemove (window ou self) pour CSS vars souris
  void AttachMouseMoveListener(blink::Element* el,
                                const LocalAction& action);

  // Câble un listener click local (sans EVENT serveur)
  void AttachClickLocalListener(blink::Element* el,
                                 const LocalAction& action);

  // ── Applicateurs d'effets ─────────────────────────────────────────────────

  // Applique / révoque un effet sur un élément selon la LocalAction
  void ApplyEffect(blink::Element* el, const LocalAction& action, bool active);

  // Met à jour une CSS custom property sur l'élément (ou :root pour global)
  void SetCssVar(blink::Element* el,
                 const std::string& var_name,
                 const std::string& value);

  // Ajoute / retire une classe CSS
  void ToggleClass(blink::Element* el,
                   const std::string& class_name,
                   bool add);

  // Applique un style inline "prop:val"
  void ApplyInlineStyle(blink::Element* el, const std::string& prop_val);

  // ── Membres ───────────────────────────────────────────────────────────────

  blink::Document* document_;  // non-owned, valide pendant toute la session

  // Permet l'annulation des tâches postées si le runner est détruit
  base::WeakPtrFactory<NhtmlLocalActionRunner> weak_factory_{this};
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_LOCAL_ACTION_RUNNER_H_
