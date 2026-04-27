# 📜 Changelog NHTML

## v0.4.0 (April 2027) — "Industrial Launch"
### Added
- **Multi-Platform Releases**: Automated builds for Windows, Linux, and macOS.
- **Premium Showcase MVC**: A complete flagship application with real-time inventory and dashboard.
- **Bilingual Documentation**: Full English and French documentation suite.
- **Compression Zstd (Active)**: B-TREE snapshots are now compressed by default (~80% size reduction).
- **Auto-Injection Bridge**: Gateway automatically injects `bridge.js` + `fzstd.js` into `.nhtml` files.
- **Integrated DevTools**: Real-time packet inspection and session time-travel replay.
- **SQLite Persistence**: Local database support for session management and showcase data.

### Improved
- **NBPS v0.4.0 Protocol**: Universal 5-byte header and strict version tracking for atomic patches.
- **Performance**: DOM mutations now processed in <1ms on the client side.
- **Security**: AGPL-3.0 License applied across the entire ecosystem.

---

## v0.3.1 (April 2026)
### Breaking Changes
- **Header Upgrade**: 3 bytes → 5 bytes (`Length` is now `u32`).
- **OpCode Realignment**: Standardized operation codes for the NBPS protocol.

### Added
- **0x10 LOG**: Binary log relay from server to browser console.
- **Supervisor**: Gateway now automatically manages the PHP backend lifecycle.

---

## v0.2.x
- **Baseline**: Initial binary NBPS protocol support.
