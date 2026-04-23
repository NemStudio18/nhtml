#include "content/renderer/nhtml/nhtml_event_listener.h"

#include "content/common/nhtml/nhtml_constants.h"
#include "content/renderer/nhtml/nhtml_socket_writer.h"
#include "third_party/blink/renderer/core/dom/events/event.h"
#include "third_party/blink/renderer/core/events/keyboard_event.h"
#include "third_party/blink/renderer/core/events/mouse_event.h"
#include "third_party/blink/renderer/core/html/forms/html_form_element.h"
#include "third_party/blink/renderer/core/html/forms/html_input_element.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"

namespace nhtml {

NhtmlEventListener::NhtmlEventListener(uint16_t           node_id,
                                       uint8_t            event_type,
                                       NhtmlSocketWriter* socket)
    : node_id_(node_id), event_type_(event_type), socket_(socket)
{
  // Mapper le code proto → nom d'event DOM pour removeEventListener
  switch (event_type) {
    case kEvtClick:   event_name_ = AtomicString("click");   break;
    case kEvtInput:   event_name_ = AtomicString("input");   break;
    case kEvtSubmit:  event_name_ = AtomicString("submit");  break;
    case kEvtKeydown: event_name_ = AtomicString("keydown"); break;
    case kEvtScroll:  event_name_ = AtomicString("scroll");  break;
    default:          event_name_ = AtomicString("custom");  break;
  }
}

void NhtmlEventListener::Invoke(blink::ExecutionContext*,
                                blink::Event* event)
{
  // Toujours sur le main thread — Blink garantit ça
  DCHECK(IsMainThread());

  switch (event_type_) {

    // ── CLICK ────────────────────────────────────────────────────────────
    case kEvtClick: {
      // Payload minimal : coordonnées relatives (optionnel mais utile)
      auto* mouse = blink::DynamicTo<blink::MouseEvent>(event);
      if (mouse) {
        uint8_t buf[8];
        int32_t x = static_cast<int32_t>(mouse->offsetX());
        int32_t y = static_cast<int32_t>(mouse->offsetY());
        // Big-endian 4B + 4B
        buf[0] = (x >> 24) & 0xFF; buf[1] = (x >> 16) & 0xFF;
        buf[2] = (x >> 8)  & 0xFF; buf[3] =  x        & 0xFF;
        buf[4] = (y >> 24) & 0xFF; buf[5] = (y >> 16) & 0xFF;
        buf[6] = (y >> 8)  & 0xFF; buf[7] =  y        & 0xFF;
        socket_->SendEvent(node_id_, kEvtClick,
            base::make_span(buf, sizeof(buf)));
      } else {
        socket_->SendEvent(node_id_, kEvtClick, {});
      }
      break;
    }

    // ── INPUT ─────────────────────────────────────────────────────────────
    case kEvtInput: {
      auto* input = blink::DynamicTo<blink::HTMLInputElement>(
          event->target()->ToNode());
      if (!input) break;

      WTFString value = input->value();
      std::string utf8 = value.Utf8();
      socket_->SendEvent(
          node_id_, kEvtInput,
          base::as_bytes(base::make_span(utf8.data(), utf8.size())));
      break;
    }

    // ── SUBMIT ────────────────────────────────────────────────────────────
    case kEvtSubmit: {
      auto* form = blink::DynamicTo<blink::HTMLFormElement>(
          event->target()->ToNode());
      if (!form) break;

      // Bloquer la soumission HTTP native
      event->preventDefault();

      // Sérialiser les champs du formulaire en paires key=value UTF-8
      // Format payload : [2B count][ [1B klen][key][2B vlen][value] × N ]
      std::vector<uint8_t> payload;

      auto* entry_list = form->ConstructEntryList(nullptr, nullptr);
      if (entry_list) {
        uint16_t count = static_cast<uint16_t>(entry_list->size());
        payload.push_back(count >> 8);
        payload.push_back(count & 0xFF);

        for (const auto& entry : *entry_list) {
          std::string key  = entry->name().Utf8();
          std::string val  = entry->Value().Utf8();

          payload.push_back(static_cast<uint8_t>(key.size()));
          payload.insert(payload.end(), key.begin(), key.end());

          uint16_t vlen = static_cast<uint16_t>(val.size());
          payload.push_back(vlen >> 8);
          payload.push_back(vlen & 0xFF);
          payload.insert(payload.end(), val.begin(), val.end());
        }
      }

      socket_->SendEvent(node_id_, kEvtSubmit,
          base::make_span(payload));
      break;
    }

    // ── KEYDOWN ───────────────────────────────────────────────────────────
    case kEvtKeydown: {
      auto* kb = blink::DynamicTo<blink::KeyboardEvent>(event);
      if (!kb) break;

      uint16_t keycode = static_cast<uint16_t>(kb->keyCode());
      uint8_t  buf[2]  = {
          static_cast<uint8_t>(keycode >> 8),
          static_cast<uint8_t>(keycode & 0xFF)
      };
      socket_->SendEvent(node_id_, kEvtKeydown,
          base::make_span(buf, sizeof(buf)));
      break;
    }

    // ── SCROLL ────────────────────────────────────────────────────────────
    case kEvtScroll: {
      // Payload : position Y de scroll (uint32 big-endian)
      auto* el = blink::DynamicTo<blink::Element>(event->target()->ToNode());
      if (!el) break;

      uint32_t scroll_y = static_cast<uint32_t>(el->scrollTop());
      uint8_t  buf[4]   = {
          static_cast<uint8_t>(scroll_y >> 24),
          static_cast<uint8_t>(scroll_y >> 16),
          static_cast<uint8_t>(scroll_y >> 8),
          static_cast<uint8_t>(scroll_y & 0xFF)
      };
      socket_->SendEvent(node_id_, kEvtScroll,
          base::make_span(buf, sizeof(buf)));
      break;
    }

    default:
      LOG(WARNING) << "[Nhtml] NhtmlEventListener: event_type inconnu 0x"
                   << std::hex << static_cast<int>(event_type_);
  }
}

}  // namespace nhtml