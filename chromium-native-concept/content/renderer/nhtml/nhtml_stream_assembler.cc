#include "content/renderer/nhtml/nhtml_stream_assembler.h"

#include <algorithm>
#include <cstring>

#include "base/check.h"
#include "content/common/nhtml/nhtml_constants.h"

namespace nhtml {

NhtmlStreamAssembler::NhtmlStreamAssembler(size_t initial_capacity) {
  buffer_.resize(initial_capacity);
}

std::vector<NhtmlPacket> NhtmlStreamAssembler::Push(
    const uint8_t* data, size_t length)
{
  EnsureCapacity(length);

  // Copier les nouveaux bytes dans le buffer
  std::memcpy(buffer_.data() + write_pos_, data, length);
  write_pos_ += length;

  std::vector<NhtmlPacket> packets;
  while (auto pkt = TryConsume()) {
    packets.push_back(std::move(*pkt));
  }
  return packets;
}

std::optional<NhtmlPacket> NhtmlStreamAssembler::TryConsume() {
  switch (state_) {

    case State::kWaitingHeader: {
      // B-TREE nécessite 5B de header (1 type + 4 length)
      // Tous les autres : 3B (1 type + 2 length)
      uint8_t peeked_type = PeekType();
      size_t hdr_size = (peeked_type == kPktBTree) ? 5 : 3;

      if (Available() < hdr_size)
        return std::nullopt;  // Attendre

      uint8_t ptype = ReadU8();
      size_t length = (ptype == kPktBTree)
          ? ReadU32BE()
          : ReadU16BE();

      pending_hdr_ = PendingHeader{ ptype, length };
      state_ = State::kWaitingPayload;

      // Enchaîner immédiatement sur kWaitingPayload
      [[fallthrough]];
    }

    case State::kWaitingPayload: {
      DCHECK(pending_hdr_.has_value());
      size_t expected = pending_hdr_->length;

      if (Available() < expected)
        return std::nullopt;  // Attendre sans toucher au buffer

      NhtmlPacket pkt;
      pkt.type    = pending_hdr_->type;
      pkt.payload = ReadBytes(expected);

      pending_hdr_.reset();
      state_ = State::kWaitingHeader;

      // Compacter le buffer si plus de la moitié est consommée
      if (read_pos_ > buffer_.size() / 2)
        Compact();

      return pkt;
    }
  }
  return std::nullopt;
}

// ─── Helpers ───────────────────────────────────────────────────────────────

size_t NhtmlStreamAssembler::pending_bytes() const {
  return write_pos_ - read_pos_;
}

size_t NhtmlStreamAssembler::Available() const {
  return write_pos_ - read_pos_;
}

uint8_t NhtmlStreamAssembler::PeekType() const {
  if (Available() == 0) return 0;
  return buffer_[read_pos_];
}

uint8_t NhtmlStreamAssembler::ReadU8() {
  DCHECK(Available() >= 1);
  return buffer_[read_pos_++];
}

uint16_t NhtmlStreamAssembler::ReadU16BE() {
  DCHECK(Available() >= 2);
  uint16_t val = (static_cast<uint16_t>(buffer_[read_pos_])     << 8)
               | (static_cast<uint16_t>(buffer_[read_pos_ + 1]));
  read_pos_ += 2;
  return val;
}

uint32_t NhtmlStreamAssembler::ReadU32BE() {
  DCHECK(Available() >= 4);
  uint32_t val = (static_cast<uint32_t>(buffer_[read_pos_])     << 24)
               | (static_cast<uint32_t>(buffer_[read_pos_ + 1]) << 16)
               | (static_cast<uint32_t>(buffer_[read_pos_ + 2]) << 8)
               | (static_cast<uint32_t>(buffer_[read_pos_ + 3]));
  read_pos_ += 4;
  return val;
}

std::vector<uint8_t> NhtmlStreamAssembler::ReadBytes(size_t n) {
  DCHECK(Available() >= n);
  std::vector<uint8_t> out(
      buffer_.begin() + read_pos_,
      buffer_.begin() + read_pos_ + n);
  read_pos_ += n;
  return out;
}

void NhtmlStreamAssembler::Compact() {
  size_t remaining = Available();
  if (remaining > 0) {
    std::memmove(buffer_.data(),
                 buffer_.data() + read_pos_,
                 remaining);
  }
  write_pos_ = remaining;
  read_pos_  = 0;
}

void NhtmlStreamAssembler::EnsureCapacity(size_t needed) {
  size_t free_space = buffer_.size() - write_pos_;
  if (free_space >= needed) return;

  // Compacter d'abord : récupérer l'espace consommé
  Compact();
  free_space = buffer_.size() - write_pos_;

  if (free_space < needed) {
    // Realloc : doubler ou ajouter le nécessaire
    size_t new_size = std::max(buffer_.size() * 2,
                               write_pos_ + needed);
    buffer_.resize(new_size);
  }
}

}  // namespace nhtml