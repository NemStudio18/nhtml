#ifndef CONTENT_RENDERER_NHTML_NHTML_INTERPRETER_H_
#define CONTENT_RENDERER_NHTML_NHTML_INTERPRETER_H_

#include <cstddef>
#include <cstdint>
#include <memory>
#include <vector>

#include "base/memory/scoped_refptr.h"
#include "base/task/single_thread_task_runner.h"
#include "content/common/nhtml/nhtml_types.h"
#include "content/renderer/nhtml/nhtml_stream_assembler.h"

namespace blink {
class Document;
}

namespace nhtml {

class NhtmlNodeTable;
class NhtmlSocketWriter;

// ─── NhtmlInterpreter ─────────────────────────────────────────────────────
//
// Chef d'orchestre du runtime Nhtml côté Renderer.
//
// Thread model :
//   IO Thread  → PushBytes(), décodage binaire, décompression B-TREE
//   Main Thread → BuildFromTree(), ApplyPatchOp(), FlushPendingStyles()
//
// Ownership :
//   NhtmlDocumentLoader owns NhtmlInterpreter
//   NhtmlInterpreter owns NhtmlNodeTable et NhtmlSocketWriter

class NhtmlInterpreter {
 public:
  NhtmlInterpreter(
      blink::Document*                                   document,
      scoped_refptr<base::SingleThreadTaskRunner>        main_runner,
      scoped_refptr<base::SingleThreadTaskRunner>        io_runner);

  ~NhtmlInterpreter();

  // Appelé depuis l'IO thread à chaque chunk réseau reçu
  void PushBytes(const uint8_t* data, size_t length);

  // Appelé quand le flux est fermé proprement par le serveur
  void OnStreamClosed();

 private:
  // IO thread : dispatch selon le type de paquet
  void DispatchPacket(NhtmlPacket packet);

  // IO thread : parse un paquet BIND v0.2 (0x04)
  bool ParseBindPacket(const std::vector<uint8_t>& payload, BindEntry& out);

  // IO thread : décompresse et parse un B-TREE (0x07)
  ParsedTree DecompressAndParse(const std::vector<uint8_t>& payload);

  // IO thread : décode les PatchOps d'un paquet PATCH (0x02)
  std::vector<PatchOp> DecodePatchOps(const std::vector<uint8_t>& payload);

  // Main thread : applique un batch de PatchOp sur le DOM Blink
  void ApplyPatchOpsOnMainThread(std::vector<PatchOp> ops);

  // Main thread : enregistre un BIND et câble les listeners natifs
  void ApplyBindOnMainThread(BindEntry entry);

  // ── Membres ───────────────────────────────────────────────────────────

  scoped_refptr<base::SingleThreadTaskRunner> main_runner_;
  scoped_refptr<base::SingleThreadTaskRunner> io_runner_;

  NhtmlStreamAssembler              assembler_;

  // Owned — créés dans le constructeur
  std::unique_ptr<NhtmlSocketWriter> socket_writer_;
  std::unique_ptr<NhtmlNodeTable>    node_table_;

  // Niveau de backpressure actuel (0=nominal, 1=ralentir, 2=urgence)
  uint8_t backpressure_level_ = 0;
};

}  // namespace nhtml

#endif  // CONTENT_RENDERER_NHTML_NHTML_INTERPRETER_H_