// content/renderer/nhtml/nhtml_socket_writer.cc
#include "content/renderer/nhtml/nhtml_socket_writer.h"

#include <cstdint>
#include <string>
#include <utility>
#include <vector>

#include "base/functional/bind.h"
#include "base/location.h"
#include "base/logging.h"
#include "content/common/nhtml/nhtml_types.h"

namespace nhtml {

NhtmlSocketWriter::NhtmlSocketWriter(
    scoped_refptr<base::SingleThreadTaskRunner> io_runner)
    : io_runner_(std::move(io_runner)) {}

// ─── SendEvent — spec v0.2 ────────────────────────────────────────────────
//
// Format paquet 0x03 EVENT :
//   [0x03][2B LENGTH][2B source_id][1B event_type]
//   [1B handler_len][... handler][2B payload_len][... payload]
//
// Le handler (ex: "panier.ajouter") est inclus dans la trame binaire
// conformément à la spec v0.2, permettant au Gateway de dispatcher
// directement sans maintenir une table côté serveur par session.

void NhtmlSocketWriter::SendEvent(uint16_t node_id,
                                  uint8_t event_type,
                                  const std::string& handler,
                                  base::span<const uint8_t> payload) {
  // Validation de sécurité
  if (handler.size() > 255) {
    LOG(ERROR) << "[NHTML] Handler name too long, dropping EVENT.";
    return;
  }

  const uint8_t handler_len  = static_cast<uint8_t>(handler.size());
  const uint16_t payload_len = static_cast<uint16_t>(payload.size());

  // Taille du payload du paquet (tout ce qui suit le header [type][2B length])
  // = 2 (source_id) + 1 (event_type) + 1 (handler_len) + handler.size()
  //   + 2 (payload_len) + payload.size()
  const uint16_t pkt_length = static_cast<uint16_t>(
      2 + 1 + 1 + handler.size() + 2 + payload.size());

  std::vector<uint8_t> packet;
  packet.reserve(1 + 2 + pkt_length);

  // Header
  packet.push_back(kPktEvent);
  packet.push_back(static_cast<uint8_t>(pkt_length >> 8));
  packet.push_back(static_cast<uint8_t>(pkt_length & 0xFF));

  // source_id
  packet.push_back(static_cast<uint8_t>(node_id >> 8));
  packet.push_back(static_cast<uint8_t>(node_id & 0xFF));

  // event_type
  packet.push_back(event_type);

  // handler_len + handler
  packet.push_back(handler_len);
  for (char c : handler) {
    packet.push_back(static_cast<uint8_t>(c));
  }

  // payload_len + payload
  packet.push_back(static_cast<uint8_t>(payload_len >> 8));
  packet.push_back(static_cast<uint8_t>(payload_len & 0xFF));
  packet.insert(packet.end(), payload.begin(), payload.end());

  io_runner_->PostTask(
      FROM_HERE,
      base::BindOnce(&NhtmlSocketWriter::WriteOnIOThread,
                     base::Unretained(this), std::move(packet)));
}

// ─── SendPong — réponse keepalive ─────────────────────────────────────────
//
// Format paquet 0x06 PING/PONG :
//   [0x06][0x00 0x02][1B direction=0x01 (PONG)][1B sequence]

void NhtmlSocketWriter::SendPong(uint8_t sequence) {
  std::vector<uint8_t> packet = {
      kPktPing,
      0x00, 0x02,   // length = 2
      0x01,         // direction = PONG
      sequence
  };
  io_runner_->PostTask(
      FROM_HERE,
      base::BindOnce(&NhtmlSocketWriter::WriteOnIOThread,
                     base::Unretained(this), std::move(packet)));
}

// ─── SendError — paquet 0x08 ERR ──────────────────────────────────────────
//
// Format :
//   [0x08][2B LENGTH][1B severity][1B origin=0x02 client]
//   [1B error_code][2B ref_id][2B msg_len][... message]

void NhtmlSocketWriter::SendError(uint8_t severity,
                                  uint8_t error_code,
                                  uint16_t ref_id,
                                  const std::string& message) {
  const uint16_t msg_len = static_cast<uint16_t>(message.size());
  // payload = 1 (severity) + 1 (origin) + 1 (error_code)
  //           + 2 (ref_id) + 2 (msg_len) + msg_len
  const uint16_t pkt_length = static_cast<uint16_t>(7 + msg_len);

  std::vector<uint8_t> packet;
  packet.reserve(1 + 2 + pkt_length);

  packet.push_back(kPktErr);
  packet.push_back(static_cast<uint8_t>(pkt_length >> 8));
  packet.push_back(static_cast<uint8_t>(pkt_length & 0xFF));
  packet.push_back(severity);
  packet.push_back(0x02);  // origin = client
  packet.push_back(error_code);
  packet.push_back(static_cast<uint8_t>(ref_id >> 8));
  packet.push_back(static_cast<uint8_t>(ref_id & 0xFF));
  packet.push_back(static_cast<uint8_t>(msg_len >> 8));
  packet.push_back(static_cast<uint8_t>(msg_len & 0xFF));
  for (char c : message) {
    packet.push_back(static_cast<uint8_t>(c));
  }

  io_runner_->PostTask(
      FROM_HERE,
      base::BindOnce(&NhtmlSocketWriter::WriteOnIOThread,
                     base::Unretained(this), std::move(packet)));
}

// ─── WriteOnIOThread — envoi réseau réel ──────────────────────────────────

void NhtmlSocketWriter::WriteOnIOThread(std::vector<uint8_t> packet) {
  DCHECK(io_runner_->BelongsToCurrentThread());

  // TODO: brancher le handle WebTransport/WebSocket injecté au démarrage.
  // Ex: transport_->Write(packet.data(), packet.size());

  VLOG(1) << "[NHTML] Writing " << packet.size() << " bytes to socket.";
}

}  // namespace nhtml
