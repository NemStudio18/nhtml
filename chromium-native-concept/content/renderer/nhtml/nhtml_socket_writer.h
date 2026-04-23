#ifndef CONTENT_RENDERER_NHTML_NHTML_SOCKET_WRITER_H_
#define CONTENT_RENDERER_NHTML_NHTML_SOCKET_WRITER_H_

#include <cstdint>
#include <string>
#include <vector>

#include "base/containers/span.h"
#include "base/memory/scoped_refptr.h"
#include "base/task/single_thread_task_runner.h"

namespace nhtml {

// ─── NhtmlSocketWriter ────────────────────────────────────────────────────
//
// Thread-safe : peut être appelé depuis le main thread (via EventListener)
// ou depuis l'IO thread. Toutes les écritures réelles sont postées vers
// l'IO thread pour rester non-bloquantes.

class NhtmlSocketWriter {
 public:
  explicit NhtmlSocketWriter(
      scoped_refptr<base::SingleThreadTaskRunner> io_runner);
  ~NhtmlSocketWriter() = default;

  // Sérialise un paquet EVENT (0x03) et l'envoie via le socket.
  // Thread-safe — peut être appelé depuis n'importe quel thread.
  // Le handler (ex: "panier.ajouter") est inclus dans la trame binaire
  // conformément à la spec v0.2.
  void SendEvent(uint16_t                     node_id,
                 uint8_t                      event_type,
                 const std::string&           handler,
                 base::span<const uint8_t>    payload);

  // Envoie un PONG en réponse à un PING du serveur (0x06).
  void SendPong(uint8_t sequence);

  // Sérialise et envoie un paquet ERR (0x08) au serveur.
  void SendError(uint8_t  severity,
                 uint8_t  error_code,
                 uint16_t ref_id,
                 const std::string& message);

 private:
  // Écriture réelle — s'exécute sur l'IO thread
  void WriteOnIOThread(std::vector<uint8_t> packet);

  scoped_refptr<base::SingleThreadTaskRunner> io_runner_;

  // Handle vers le WebTransport natif (initialisé par NhtmlInterpreter)
  // Dans un vrai fork : blink::WebTransport* ou network::mojom::WebTransport
  // Pour le scaffold : abstrait derrière une interface minimale
  // TODO: injecter le handle WebTransport réel lors de l'intégration
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_SOCKET_WRITER_H_