#ifndef CONTENT_BROWSER_LOADER_NHTML_NHTML_URL_LOADER_THROTTLE_H_
#define CONTENT_BROWSER_LOADER_NHTML_NHTML_URL_LOADER_THROTTLE_H_

#include "third_party/blink/public/common/loader/url_loader_throttle.h"

namespace nhtml {

// ─── NhtmlURLLoaderThrottle ────────────────────────────────────────────────
//
// Inséré dans la chaîne de throttles du Browser Process.
// Son unique rôle : détecter le MIME type application/x-nhtml-stream
// et injecter un header X-Nhtml-Intercept pour signaler au Renderer
// qu'il doit activer NhtmlDocumentLoader au lieu du parser HTML standard.
//
// Point d'insertion dans Chromium :
//   content/browser/loader/navigation_url_loader_impl.cc
//   → NavigationURLLoaderImpl::CreateThrottles()
//   → Ajouter : throttles.push_back(
//       std::make_unique<NhtmlURLLoaderThrottle>());

class NhtmlURLLoaderThrottle : public blink::URLLoaderThrottle {
 public:
  NhtmlURLLoaderThrottle() = default;
  ~NhtmlURLLoaderThrottle() override = default;

  // Appelé quand les headers de réponse sont disponibles,
  // avant que le body ne commence à arriver.
  void WillProcessResponse(
      const GURL& response_url,
      network::mojom::URLResponseHead* response_head,
      bool* defer) override;

  const char* NameForLogging() override { return "NhtmlURLLoaderThrottle"; }
};

}  // namespace nhtml

#endif  // CONTENT_BROWSER_LOADER_NHTML_NHTML_URL_LOADER_THROTTLE_H_