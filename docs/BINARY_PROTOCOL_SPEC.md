# 🛰️ NHTML Binary Protocol Specification (NBPS) v0.2.2
**Version Stable : 23 Avril 2026**

## 1. Types de Données Standard
- `u8`, `u16`, `u32` : Entiers non-signés (Big-Endian).
- `string8` : `[Length: u8] [UTF-8 Bytes...]` (Max 255 chars).
- `string16` : `[Length: u16] [UTF-8 Bytes...]` (Max 65535 chars).

---

## 2. Types de Paquets (OpCodes)

### 0x01 : HELLO (Handshake)
- **Client -> Serveur** : `[0x01] [LastVersion: u32] [SessionID: string8]`
- **Serveur -> Client** : `[0x01] [KeepAlive: u16] [SessionID: string8]`

### 0x02 : EVENT (Interaction)
- **Client -> Serveur** : `[0x02] [NodeID: u32]`
  *Sécurité P3 : Le Gateway valide que le NodeID appartient à la session.*

### 0x03 : PATCH (Mutation DOM)
- **Serveur -> Client** : `[0x03] [OpCount: u16] [OpEntries...]`
- **OpEntry** : `[TargetID: u16] [OpType: u8] [NodeVersion: u32] [Data...]`

**OpTypes :**
- `0x01` (SetText) : `[Value: string16]`
- `0x02` (SetAttr) : `[Key: string8] [Value: string16]`
- `0x04` (AddClass) : `[Class: string8]`
- `0x05` (DelClass) : `[Class: string8]`
- `0x09` (SetStyle) : `[Prop: string8] [Value: string16]`
- `0x0A` (ReplaceInner) : `[HTML: string16]`

### 0x04 : SYNC (Resync rapide)
- **Serveur -> Client** : `[0x04] [GlobalVersion: u32]`

### 0x05 : BTREE (Full Snapshot / Recovery)
- **Serveur -> Client** : `[0x05] [TotalLen: u32] [Compression: u8] [OrigLen: u32] [Checksum: u32] [Payload]`
  - `Compression` : 0x00 (None), 0x01 (Zstd).
  - `Payload` : `[NodeCount: u16] [ (NodeID: u16, Version: u32, Val: string16) ... ]`

### 0x06 : PING / PONG
- **Structure** : `[0x06] [Sequence: u8]`

---

## 3. Sécurité & Intégrité (P3)
1. **Validation de Session** : Chaque paquet `0x02` est vérifié contre la table de session SQLite.
2. **CRC32** : Les paquets `0x05` incluent un checksum pour éviter la corruption du DOM.
3. **Session Enforcement** : Le SessionID doit être un UUID v4 valide fourni lors du HELLO.

---
**Status :** INDUSTRIEL. Cette spécification est la source de vérité pour le Gateway Rust, le Polyfill JS/WASM et le SDK PHP.
