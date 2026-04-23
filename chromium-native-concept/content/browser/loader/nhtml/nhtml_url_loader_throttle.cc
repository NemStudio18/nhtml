#include "content/browser/loader/nhtml/nhtml_url_loader_throttle.h"

#include "content/common/nhtml/nhtml_constants.h"
#include "services/network/public/mojom/url_response_head.mojom.h"

namespace nhtml {

void NhtmlURLLoaderThrottle::WillProcessResponse(
    const GURL& response_url,
    network::mojom::URLResponseHead* response_head,
    bool* defer)
{
  // Sécurité : ne rien faire si les headers sont absents
  if (!response_head || !response_head->headers)
    return;

  const std::string& mime = response_head->mime_type;

  if (mime != kNhtmlMimeType)
    return;

  // Vérifier la version du protocole
  std::string version;
  response_head->headers->GetNormalizedHeader(
      kNhtmlVersionHeader, &version);

  if (version != kNhtmlVersion) {
    // Version incompatible : on laisse Blink gérer normalement
    // plutôt que de crasher silencieusement
    LOG(WARNING) << "[Nhtml] Version mismatch: got "
                 << version << ", expected " << kNhtmlVersion
                 << " — falling back to standard loader";
    return;
  }

  // Tout est bon : signaler au Renderer via un header synthétique
  // Ce header n'est jamais envoyé par le réseau — on l'injecte ici
  response_head->headers->SetHeader(kNhtmlInterceptHeader, "1");

  LOG(INFO) << "[Nhtml] Stream détecté sur " << response_url.spec()
            << " — mode souverain activé";

  // On ne defer pas (*defer reste false) :
  // le body doit commencer à arriver immédiatement
  // NhtmlDocumentLoader côté Renderer prendra le relais
}

}  // namespace nhtml