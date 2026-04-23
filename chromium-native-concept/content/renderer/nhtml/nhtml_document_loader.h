#ifndef CONTENT_RENDERER_NHTML_NHTML_DOCUMENT_LOADER_H_
#define CONTENT_RENDERER_NHTML_NHTML_DOCUMENT_LOADER_H_

#include "third_party/blink/renderer/core/loader/document_loader.h"

namespace nhtml {

class NhtmlInterpreter;

// ─── NhtmlDocumentLoader ──────────────────────────────────────────────────
//
// Sous-classe de blink::DocumentLoader.
// Active le "mode souverain" Nhtml en interceptant CommitData()
// avant que le HTML parser ne reçoive les bytes.
//
// Activation :
//   Détecte le header X-Nhtml-Intercept: 1 injecté par
//   NhtmlURLLoaderThrottle côté Browser Process.
//
// Quand is_nhtml_stream_ == true :
//   → CommitData() redirige TOUS les bytes vers NhtmlInterpreter
//   → Le HTML parser de Blink ne reçoit RIEN
//   → La page est entièrement construite via le protocole binaire

class NhtmlDocumentLoader final : public blink::DocumentLoader {
 public:
  // Factory — retourne nullptr si le flux n'est pas Nhtml
  // (permet un fallback propre vers DocumentLoader standard)
  static NhtmlDocumentLoader* CreateIfNhtml(
      blink::LocalFrame* frame,
      const blink::WebNavigationParams& params);

  ~NhtmlDocumentLoader() override;

 protected:
  // Point d'intercepton principal
  // Appelé par Blink à chaque chunk de données réseau disponible
  void CommitData(const char* bytes, size_t length) override;

  // Appelé quand la réponse complète est reçue
  void FinishedLoading(base::TimeTicks finish_time) override;

 private:
  NhtmlDocumentLoader(blink::LocalFrame* frame,
                      const blink::WebNavigationParams& params);

  bool                              is_nhtml_stream_ = false;
  std::unique_ptr<NhtmlInterpreter> interpreter_;
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_DOCUMENT_LOADER_H_