#ifndef CONTENT_RENDERER_NHTML_NHTML_EVENT_LISTENER_H_
#define CONTENT_RENDERER_NHTML_NHTML_EVENT_LISTENER_H_

#include <cstdint>
#include <string>

#include "third_party/blink/renderer/core/dom/events/native_event_listener.h"

namespace nhtml {

class NhtmlSocketWriter;

// ─── NhtmlEventListener ───────────────────────────────────────────────────
//
// Listener natif C++ — sous-classe de blink::NativeEventListener.
// NE PASSE JAMAIS par V8 ni par le système d'événements JavaScript.
//
// Cycle de vie :
//   - Créé avec MakeGarbageCollected<> → géré par Oilpan
//   - Stocké dans NhtmlNodeEntry::listeners via Persistent<>
//   - Libéré automatiquement quand Unbind() retire le Persistent<>
//
// Quand Blink dispatch un événement sur un nœud bindé, Invoke() est
// appelé sur le main thread. On sérialise immédiatement le paquet
// EVENT (0x03) et on le PostTask vers l'IO thread pour écriture socket.

class NhtmlEventListener final : public blink::NativeEventListener {
 public:
  NhtmlEventListener(uint16_t          node_id,
                     uint8_t           event_type,
                     NhtmlSocketWriter* socket);

  // Point d'entrée unique — appelé par Blink sur le main thread
  void Invoke(blink::ExecutionContext* context,
              blink::Event* event) override;

  // Nécessaire pour removeEventListener()
  const AtomicString& EventName() const { return event_name_; }

 private:
  uint16_t           node_id_;
  uint8_t            event_type_;
  AtomicString       event_name_;  // "click", "input", etc.
  NhtmlSocketWriter* socket_;      // non-owning
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_EVENT_LISTENER_H_