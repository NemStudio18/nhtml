#ifndef CONTENT_RENDERER_NHTML_NHTML_STREAM_ASSEMBLER_H_
#define CONTENT_RENDERER_NHTML_NHTML_STREAM_ASSEMBLER_H_

#include <cstdint>
#include <optional>
#include <vector>

#include "content/common/nhtml/nhtml_types.h"

namespace nhtml {

// ─── NhtmlStreamAssembler ──────────────────────────────────────────────────
//
// Reconstitue des paquets Nhtml complets depuis un flux binaire fragmenté.
// Tourne EXCLUSIVEMENT sur l'IO thread.
//
// Invariant fondamental :
//   Un paquet n'est jamais retourné tant que tous ses bytes ne sont pas
//   disponibles dans le buffer interne. Zéro état corrompu possible.
//
// Format des paquets :
//   Standards  : [1B type][2B length_u16][...payload]
//   B-TREE(07) : [1B type][4B length_u32][...payload]  ← cas spécial

class NhtmlStreamAssembler {
 public:
  explicit NhtmlStreamAssembler(size_t initial_capacity = 65536);
  ~NhtmlStreamAssembler() = default;

  // Ajouter des bytes reçus du réseau
  // Retourne tous les paquets complets disponibles après cet ajout
  std::vector<NhtmlPacket> Push(const uint8_t* data, size_t length);

  // Nombre de bytes en attente dans le buffer
  size_t pending_bytes() const;

 private:
  enum class State {
    kWaitingHeader,
    kWaitingPayload,
  };

  struct PendingHeader {
    uint8_t type   = 0;
    size_t  length = 0;  // taille attendue du payload
  };

  // Tente de parser un paquet depuis la position courante
  // Retourne nullopt si les données sont insuffisantes
  std::optional<NhtmlPacket> TryConsume();

  // Helpers lecture
  uint8_t  PeekType() const;
  uint8_t  ReadU8();
  uint16_t ReadU16BE();
  uint32_t ReadU32BE();
  std::vector<uint8_t> ReadBytes(size_t n);

  size_t Available() const;
  void   EnsureCapacity(size_t needed);
  void   Compact();  // déplace read_pos → 0 pour réutiliser l'espace

  std::vector<uint8_t> buffer_;
  size_t               read_pos_  = 0;
  size_t               write_pos_ = 0;

  State                       state_ = State::kWaitingHeader;
  std::optional<PendingHeader> pending_hdr_;
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_STREAM_ASSEMBLER_H_