// content/renderer/nhtml/nhtml_interpreter.cc
#include "content/renderer/nhtml/nhtml_interpreter.h"

#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/functional/bind.h"
#include "base/location.h"
#include "base/logging.h"
#include "base/task/single_thread_task_runner.h"
#include "content/common/nhtml/nhtml_types.h"
#include "content/renderer/nhtml/nhtml_node_table.h"
#include "content/renderer/nhtml/nhtml_socket_writer.h"

namespace blink { class Document; }

namespace nhtml {

// ─── Helpers de lecture binaire (big-endian, safe) ────────────────────────

namespace {

// Lit un uint16_t BE depuis buf[pos]. Retourne false si débordement.
bool ReadU16(const std::vector<uint8_t>& buf, size_t& pos, uint16_t& out) {
  if (pos + 2 > buf.size()) return false;
  out = static_cast<uint16_t>((buf[pos] << 8) | buf[pos + 1]);
  pos += 2;
  return true;
}

// Lit un uint8_t depuis buf[pos].
bool ReadU8(const std::vector<uint8_t>& buf, size_t& pos, uint8_t& out) {
  if (pos >= buf.size()) return false;
  out = buf[pos++];
  return true;
}

// Lit une string préfixée par un uint8_t de longueur.
bool ReadStr8(const std::vector<uint8_t>& buf, size_t& pos, std::string& out) {
  uint8_t len = 0;
  if (!ReadU8(buf, pos, len)) return false;
  if (pos + len > buf.size()) return false;
  out.assign(reinterpret_cast<const char*>(buf.data() + pos), len);
  pos += len;
  return true;
}

// Sécurité : taille max d'un paquet unique = 10 Mo
constexpr size_t kMaxPacketSize = 10 * 1024 * 1024;

}  // namespace

// ─── Constructeur / Destructeur ────────────────────────────────────────────

NhtmlInterpreter::NhtmlInterpreter(
    blink::Document* document,
    scoped_refptr<base::SingleThreadTaskRunner> main_runner,
    scoped_refptr<base::SingleThreadTaskRunner> io_runner)
    : main_runner_(std::move(main_runner)),
      io_runner_(std::move(io_runner)),
      socket_writer_(std::make_unique<NhtmlSocketWriter>(io_runner_)),
      node_table_(std::make_unique<NhtmlNodeTable>(document)) {

  LOG(INFO) << "[NHTML] NhtmlInterpreter initialized.";

  assembler_.SetPacketCallback(base::BindRepeating(
      &NhtmlInterpreter::DispatchPacket, base::Unretained(this)));
}

NhtmlInterpreter::~NhtmlInterpreter() {
  LOG(INFO) << "[NHTML] NhtmlInterpreter destroyed.";
}

// ─── IO Thread : réception des octets réseau ──────────────────────────────

void NhtmlInterpreter::PushBytes(const uint8_t* data, size_t length) {
  DCHECK(io_runner_->BelongsToCurrentThread());
  assembler_.ProcessBytes(data, length);
}

void NhtmlInterpreter::OnStreamClosed() {
  DCHECK(io_runner_->BelongsToCurrentThread());
  LOG(INFO) << "[NHTML] Stream closed by server.";
}

// ─── IO Thread : dispatch des paquets assemblés ───────────────────────────

void NhtmlInterpreter::DispatchPacket(NhtmlPacket packet) {
  DCHECK(io_runner_->BelongsToCurrentThread());

  // Garde-fou sécurité : refuser les paquets démesurés
  if (packet.payload.size() > kMaxPacketSize) {
    LOG(ERROR) << "[NHTML] Packet too large (" << packet.payload.size()
               << " bytes), dropping.";
    socket_writer_->SendError(0x02, kErrPayloadTooLarge, 0,
                              "Packet exceeds size limit");
    return;
  }

  switch (packet.type) {

    // 0x01 HELLO — handshake serveur → client
    case kPktHello:
      LOG(INFO) << "[NHTML] HELLO received.";
      break;

    // 0x02 PATCH — mutations DOM
    case kPktPatch: {
      std::vector<PatchOp> ops = DecodePatchOps(packet.payload);
      if (!ops.empty()) {
        // Thread-safe : on transfère la propriété via std::move dans le PostTask.
        // Le vecteur ops_to_apply sera consommé exclusivement par le Main Thread.
        main_runner_->PostTask(
            FROM_HERE,
            base::BindOnce(&NhtmlInterpreter::ApplyPatchOpsOnMainThread,
                           base::Unretained(this), std::move(ops)));
      }
      break;
    }

    // 0x04 BIND — métadonnées n- pour un nœud (spec v0.2)
    case kPktBind: {
      BindEntry entry;
      if (ParseBindPacket(packet.payload, entry)) {
        main_runner_->PostTask(
            FROM_HERE,
            base::BindOnce(&NhtmlInterpreter::ApplyBindOnMainThread,
                           base::Unretained(this), std::move(entry)));
      } else {
        LOG(WARNING) << "[NHTML] Malformed BIND packet, ignoring.";
      }
      break;
    }

    // 0x06 PING — keepalive
    case kPktPing:
      // Répondre PONG : même type, direction=0x01, même sequence
      if (packet.payload.size() >= 2) {
        socket_writer_->SendPong(packet.payload[1]);
      }
      break;

    // 0x07 B-TREE — arbre DOM initial compressé
    case kPktBTree: {
      ParsedTree tree = DecompressAndParse(packet.payload);
      if (tree.has_error()) {
        LOG(ERROR) << "[NHTML] B-TREE parse error: "
                   << static_cast<int>(tree.error_code());
        socket_writer_->SendError(0x03, tree.error_code(), 0, "BTree parse failed");
      } else {
        main_runner_->PostTask(
            FROM_HERE,
            base::BindOnce(&NhtmlNodeTable::BuildFromTree,
                           base::Unretained(node_table_.get()),
                           std::move(tree)));
      }
      break;
    }

    // 0x08 ERR — erreur reçue du serveur
    case kPktErr:
      if (packet.payload.size() >= 3) {
        LOG(ERROR) << "[NHTML] ERR from server: code=0x"
                   << std::hex << static_cast<int>(packet.payload[2]);
      }
      break;

    default:
      LOG(WARNING) << "[NHTML] Unknown packet type: 0x"
                   << std::hex << static_cast<int>(packet.type);
  }
}

// ─── Parser BIND v0.2.1 ───────────────────────────────────────────────────
//
// Format : [2B target_id][1B nid_len][nid][1B selector_len][selector]
//           [1B listen_mask][1B behavior_flags][1B debounce_100ms]
//           [1B handler_len][handler][1B nmodel_len][nmodel]
//           [1B ntext_len][ntext]
//           [1B local_action_count]          ← extension v0.2.1
//           [local_action_count × LOCAL_ACTION]

bool NhtmlInterpreter::ParseBindPacket(const std::vector<uint8_t>& payload,
                                       BindEntry& out) {
  size_t pos = 0;
  if (!ReadU16(payload, pos, out.target_id))        return false;
  if (!ReadStr8(payload, pos, out.n_id))            return false;
  if (!ReadStr8(payload, pos, out.selector))        return false;
  if (!ReadU8(payload, pos, out.listen_mask))       return false;
  if (!ReadU8(payload, pos, out.behavior_flags))    return false;
  if (!ReadU8(payload, pos, out.debounce_100ms))    return false;
  if (!ReadStr8(payload, pos, out.handler))         return false;
  if (!ReadStr8(payload, pos, out.n_model))         return false;
  if (!ReadStr8(payload, pos, out.n_text))          return false;

  // ── Extension v0.2.1 : Local Actions (optionnel, 0 = compat v0.2) ────────
  uint8_t local_action_count = 0;
  if (pos < payload.size()) {
    if (!ReadU8(payload, pos, local_action_count)) return false;
  }

  out.local_actions.reserve(local_action_count);

  for (uint8_t i = 0; i < local_action_count; ++i) {
    LocalAction la;
    uint8_t type_raw = 0, trigger_raw = 0, flags_raw = 0;

    if (!ReadU8(payload, pos, type_raw))    return false;
    if (!ReadU8(payload, pos, trigger_raw)) return false;
    if (!ReadStr8(payload, pos, la.param))  return false;
    if (!ReadU8(payload, pos, flags_raw))   return false;
    if (!ReadU8(payload, pos, la.threshold_x10)) return false;

    la.type    = static_cast<LocalActionType>(type_raw);
    la.trigger = static_cast<LocalActionTrigger>(trigger_raw);
    la.flags.once          = (flags_raw >> 0) & 1;
    la.flags.reverse_leave = (flags_raw >> 1) & 1;
    la.flags.scope_self    = (flags_raw >> 2) & 1;

    out.local_actions.push_back(std::move(la));
  }

  VLOG(1) << "[NHTML] BIND node=" << out.target_id
          << " n-id=\"" << out.n_id << "\""
          << " handler=\"" << out.handler << "\""
          << " local_actions=" << out.local_actions.size();
  return true;
}

// ─── Main Thread : application des mutations ──────────────────────────────

void NhtmlInterpreter::ApplyPatchOpsOnMainThread(std::vector<PatchOp> ops) {
  DCHECK(main_runner_->BelongsToCurrentThread());
  // TODO: Implémenter node_table_->ApplyPatchOp(op) pour chaque op
  for (const auto& op : ops) {
    (void)op;  // silence unused warning pendant le scaffold
  }
}

void NhtmlInterpreter::ApplyBindOnMainThread(BindEntry entry) {
  DCHECK(main_runner_->BelongsToCurrentThread());
  // TODO: Stocker l'entry dans la NodeTable et câbler les listeners natifs Blink
  // node_table_->RegisterBind(std::move(entry));
  (void)entry;
}

// ─── Parsing B-TREE ───────────────────────────────────────────────────────

ParsedTree NhtmlInterpreter::DecompressAndParse(
    const std::vector<uint8_t>& payload) {
  // TODO: 1. Lire le header (compression, length_raw, checksum)
  //        2. Décompresser avec Zstd si compression == 0x01
  //        3. Vérifier CRC32
  //        4. Parser l'arbre en NodeSpec récursivement
  if (payload.empty()) {
    return ParsedTree::Error(kErrDecompressFail);
  }
  ParsedTree tree;
  return tree;
}

// ─── Décodage PATCH ───────────────────────────────────────────────────────

std::vector<PatchOp> NhtmlInterpreter::DecodePatchOps(
    const std::vector<uint8_t>& payload) {
  // Format : [1B op_count][ op_count × OPERATION ]
  // OPERATION : [1B op_type][2B target_id][2B op_length][... op_data]
  std::vector<PatchOp> ops;
  if (payload.empty()) return ops;

  size_t pos = 0;
  uint8_t op_count = payload[pos++];
  ops.reserve(op_count);

  for (uint8_t i = 0; i < op_count; ++i) {
    if (pos + 5 > payload.size()) break;

    PatchOp op;
    op.type      = payload[pos++];
    uint16_t tid = static_cast<uint16_t>((payload[pos] << 8) | payload[pos + 1]);
    pos += 2;
    op.target_id = tid;
    uint16_t op_len = static_cast<uint16_t>((payload[pos] << 8) | payload[pos + 1]);
    pos += 2;

    if (pos + op_len > payload.size()) break;
    // TODO: Parser op_data selon op_type (cf. spec §5)
    pos += op_len;

    ops.push_back(std::move(op));
  }
  return ops;
}

}  // namespace nhtml
