# 🛰️ NHTML Gateway

**The high-performance binary relay for NHTML applications.**

The NHTML Gateway is a WebSocket server written in Rust, designed to bridge web clients (Binary NBPS) and traditional PHP backends. It transforms binary DOM mutations into ultra-fast state requests.

## 🚀 Key Features
- **Binary Relay (NBPS)**: Native management of the binary protocol for minimal latency.
- **PHP Supervisor**: Automatically launches and monitors your PHP processes in local mode.
- **MPSC Architecture**: Robust management of multiple connections via asynchronous channels (Tokio).
- **Zstd Compression**: On-the-fly compression of B-TREE snapshots to save up to 70% bandwidth.
- **SQLite Persistence**: Automatic session archiving for replay and diagnostics.

## 📦 Installation (Binaries)
No compilation is required for standard use.
1. Download the binary corresponding to your OS from the [Releases](https://github.com/NemStudio18/nhtml/releases).
2. Place the binary in your PHP project folder.
3. Run: `./nhtml start --dev` (your app will be available at `http://127.0.0.1:3000`)

## 💻 CLI Commands

The `nhtml` binary exposes several commands to fit your workflow:

- **`nhtml start`**: Launches the WebSocket Gateway, the HTTP server (port 3000, with automatic NHTML bridge injection), and the PHP supervisor.
  - `--dev`: Enables auto-reload (live-reload).
  - `--ws-port <port>`: Force WebSocket port (default: 8080).
  - `--php-port <port>`: Force PHP backend port (default: 8000).
- **`nhtml devtools`**: Launches the diagnostic dashboard.
  - `--port <port>`: Set DevTools server port (default: 8081).
- **`nhtml bench <file.nhtml>`**: Compares transmission performance (Raw HTML size vs NBPS patch size).

## 🛠️ Development
If you wish to compile the gateway yourself:
```bash
cargo build --release
```

## 📜 License
This component is licensed under **AGPL v3**.
For commercial needs or proprietary cloud deployments, contact NemStudio.
