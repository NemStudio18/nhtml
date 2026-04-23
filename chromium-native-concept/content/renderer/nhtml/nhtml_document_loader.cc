#include "content/renderer/nhtml/nhtml_document_loader.h"

#include "base/logging.h"
#include "base/task/thread_pool.h"
#include "content/common/nhtml/nhtml_constants.h"
#include "content/renderer/nhtml/nhtml_interpreter.h"
#include "third_party/blink/public/web/web_navigation_params.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"

namespace nhtml {

// ─── Factory ───────────────────────────────────────────────────────────────

// static
NhtmlDocumentLoader* NhtmlDocumentLoader::CreateIfNhtml(
    blink::LocalFrame* frame,
    const blink::WebNavigationParams& params)
{
  // Vérifier le header injecté par NhtmlURLLoaderThrottle
  // Si absent → retourner nullptr → Blink utilise DocumentLoader standard
  if (!params.response.HttpHeaderField(
          AtomicString(kNhtmlInterceptHeader)).Contains("1")) {
    return nullptr;
  }

  LOG(INFO) << "[Nhtml] NhtmlDocumentLoader activé — mode souverain";
  return new NhtmlDocumentLoader(frame, params);
}

// ─── Constructeur ──────────────────────────────────────────────────────────

NhtmlDocumentLoader::NhtmlDocumentLoader(
    blink::LocalFrame* frame,
    const blink::WebNavigationParams& params)
    : blink::DocumentLoader(frame, params)
    , is_nhtml_stream_(true)
{
  // Créer l'interpréteur avec le main thread runner et un IO thread dédié
  auto main_runner = frame->GetTaskRunner(blink::TaskType::kInternalDefault);
  auto io_runner   = base::ThreadPool::CreateSingleThreadTaskRunner(
      { base::TaskPriority::USER_BLOCKING,
        base::TaskShutdownBehavior::SKIP_ON_SHUTDOWN });

  interpreter_ = std::make_unique<NhtmlInterpreter>(
      frame->GetDocument(),
      std::move(main_runner),
      std::move(io_runner));
}

NhtmlDocumentLoader::~NhtmlDocumentLoader() = default;

// ─── CommitData ────────────────────────────────────────────────────────────
//
// C'est ici que le "mode souverain" prend effet.
// En mode Nhtml : les bytes n'arrivent JAMAIS au HTML parser.
// En mode fallback : comportement Blink standard intact.

void NhtmlDocumentLoader::CommitData(const char* bytes, size_t length) {
  if (!is_nhtml_stream_) {
    // Fallback : déléguer au parser HTML normal
    blink::DocumentLoader::CommitData(bytes, length);
    return;
  }

  // Mode souverain : rediriger le flux binaire vers l'interpréteur
  // NhtmlInterpreter::PushBytes() s'exécutera sur l'IO thread
  interpreter_->PushBytes(
      reinterpret_cast<const uint8_t*>(bytes), length);
}

// ─── FinishedLoading ───────────────────────────────────────────────────────

void NhtmlDocumentLoader::FinishedLoading(base::TimeTicks finish_time) {
  if (is_nhtml_stream_ && interpreter_) {
    // Le flux initial est terminé (B-TREE reçu)
    // La connexion WebTransport/WebSocket reste ouverte pour PATCH/EVENT
    interpreter_->OnStreamClosed();
    return;
  }
  blink::DocumentLoader::FinishedLoading(finish_time);
}

}  // namespace nhtml