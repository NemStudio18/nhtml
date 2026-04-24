// NHTML v0.3.1 JS Bridge
// Cette couche s'occupe de faire le lien entre le DOM et le moteur WASM (Polyfill).
// Gère désormais le fallback HTTP automatique et les logs serveurs.

import init, { NhtmlPolyfill } from '../pkg/nhtml_polyfill.js';

let polyfill = null;
let socket = null;
let transportMode = "WS"; // "WS", "HTTP", ou "WASM"
let phpWasmEngine = null;
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000;

window.nhtml_last_version = 0; 
window.nhtml_session_id = localStorage.getItem('nhtml_session_id') || "";

export async function initNhtml(wsUrl) {
    if (!polyfill) {
        await init();
        polyfill = new NhtmlPolyfill();
    }

    if (!window.nhtml_session_id || window.nhtml_session_id.length < 5) {
        window.nhtml_session_id = crypto.randomUUID().replace(/-/g, '');
        localStorage.setItem('nhtml_session_id', window.nhtml_session_id);
    }

    injectStyles();
    injectHud();
    
    connect(wsUrl);
}

function injectStyles() {
    if (document.getElementById('nhtml-styles')) return;
    const style = document.createElement('style');
    style.id = 'nhtml-styles';
    style.innerHTML = `
        @keyframes nhtml-glow {
            0% { background-color: rgba(0, 255, 0, 0.2); outline: 2px solid rgba(0, 255, 0, 0.5); }
            100% { background-color: transparent; outline: 2px solid transparent; }
        }
        .nhtml-patch-glow { animation: nhtml-glow 0.6s ease-out; }
        #nhtml-hud {
            position: fixed; bottom: 10px; right: 10px;
            background: rgba(0,0,0,0.8); color: #0f0;
            padding: 5px 10px; border-radius: 5px;
            font-family: monospace; font-size: 10px;
            z-index: 9999; border: 1px solid #050;
            pointer-events: none;
        }
        #nhtml-hud span { color: #fff; }
    `;
    document.head.appendChild(style);
}

function injectHud() {
    if (document.getElementById('nhtml-hud')) return;
    const hud = document.createElement('div');
    hud.id = 'nhtml-hud';
    hud.innerHTML = 'NHTML: <span id="nhtml-hud-pkts">0</span> pkts | <span id="nhtml-hud-size">0</span> B <span id="nhtml-hud-mode">(WS)</span>';
    document.body.appendChild(hud);
    window.nhtml_stats = { pkts: 0, size: 0 };
}

async function loadPhpWasm() {
    if (phpWasmEngine) return;
    console.log("%c[NHTML] 🪶 Chargement du moteur PHP-WASM (Local)...", "color: #ff00ff; font-weight: bold;");
    const { PhpWeb } = await import('./php-wasm/PhpWeb.mjs');
    phpWasmEngine = new PhpWeb();
    console.log("%c[NHTML] 🪶 PHP-WASM est prêt !", "color: #ff00ff; font-weight: bold;");
}

function applyJsonPatches(patches) {
    console.log(`%c[NHTML] 🪶 PATCH WASM Reçu | Ops: ${patches.length}`, "color: #ff00ff; font-weight: bold;");
    for (const p of patches) {
        const el = document.querySelector(`[n-id="${p.nid}"]`);
        if (!el) continue;
        
        if (p.op === 'set_text') {
            el.textContent = p.value;
            el.classList.remove('nhtml-patch-glow'); void el.offsetWidth; el.classList.add('nhtml-patch-glow');
        } else if (p.op === 'set_html') {
            el.innerHTML = p.value;
            el.classList.remove('nhtml-patch-glow'); void el.offsetWidth; el.classList.add('nhtml-patch-glow');
        } else if (p.op === 'set_class') {
            el.className = p.value;
        } else if (p.op === 'set_attr') {
            el.setAttribute(p.attr, p.value);
        } else if (p.op === 'append_html') {
            el.insertAdjacentHTML('beforeend', p.value);
        } else if (p.op === 'remove_node') {
            el.remove();
        }
    }
}

async function sendBinary(data, jsonPayload = null) {
    if (transportMode === "WASM") {
        if (!jsonPayload) return;
        const modeEl = document.getElementById('nhtml-hud-mode');
        if (modeEl) modeEl.innerText = "(WASM)";
        
        try {
            await loadPhpWasm();
            let appCode = await fetch('app.php').then(r => r.text());
            
            // Mock php://input to pass the JSON payload directly into the PHP script
            let safePayload = jsonPayload.replace(/'/g, "\\'");
            let executionCode = appCode.replace(/file_get_contents\s*\(\s*['"]php:\/\/input['"]\s*\)/g, `'${safePayload}'`);
            
            const output = await phpWasmEngine.run(executionCode);
            try {
                // Strip anything before the JSON array in case of PHP warnings/notices
                const jsonStart = output.indexOf('[');
                if (jsonStart !== -1) {
                    const patches = JSON.parse(output.substring(jsonStart));
                    applyJsonPatches(patches);
                }
            } catch (e) {
                console.error("[NHTML] 🪶 Erreur parsing JSON PHP:", output);
            }
        } catch (err) {
            console.error("[NHTML] 🪶 Erreur d'exécution PHP-WASM:", err);
        }
        return;
    }

    const type = new Uint8Array(data)[0];
    console.log(`%c[NBPS] OUT: 0x${type.toString(16).padStart(2,'0')} (${data.byteLength} bytes)`, "color: #ffaa00; font-weight: bold;");
    
    if (transportMode === "WS" && socket && socket.readyState === WebSocket.OPEN) {
        socket.send(data);
    } else {
        if (transportMode === "WS") {
            console.warn("[NHTML] Bascule en mode HTTP POST (Mutualisé)...");
            transportMode = "HTTP";
            const modeEl = document.getElementById('nhtml-hud-mode');
            if (modeEl) modeEl.innerText = "(HTTP)";
        }
        
        try {
            const response = await fetch(window.location.href, {
                method: 'POST',
                headers: { 'Content-Type': 'application/octet-stream', 'X-NHTML-Session': window.nhtml_session_id },
                body: data
            });
            
            // Si le serveur HTTP ne gère pas le POST (ex: GitHub Pages renvoie 405), on passe en WASM
            if (response.status === 405 || response.status === 404) {
                console.warn("[NHTML] Hébergement statique détecté. Bascule définitive sur PHP-WASM !");
                transportMode = "WASM";
                return sendBinary(data, jsonPayload); // Retry with WASM
            }

            const buffer = await response.arrayBuffer();
            if (buffer.byteLength > 0) {
                processMessage(new Uint8Array(buffer));
            }
        } catch (e) {
            console.warn("[NHTML] Serveur injoignable, bascule sur PHP-WASM !");
            transportMode = "WASM";
            return sendBinary(data, jsonPayload);
        }
    }
}

function processMessage(chunk) {
    if (chunk.length < 5) return;
    const view = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
    const type = chunk[0];
    console.log(`%c[NBPS] IN: 0x${type.toString(16).padStart(2,'0')} (${chunk.length} bytes)`, "color: #00ff00; font-weight: bold;");
    
    window.nhtml_stats.pkts++;
    window.nhtml_stats.size += chunk.length;
    const hudPkts = document.getElementById('nhtml-hud-pkts');
    const hudSize = document.getElementById('nhtml-hud-size');
    const hudMode = document.getElementById('nhtml-hud-mode');
    if (hudPkts) hudPkts.innerText = window.nhtml_stats.pkts;
    if (hudSize) hudSize.innerText = (window.nhtml_stats.size > 1024) ? (window.nhtml_stats.size/1024).toFixed(1) + " KB" : window.nhtml_stats.size + " B";
    if (hudMode) hudMode.innerText = `(${transportMode})`;


    if (type === 0x09) { // RELOAD
        window.location.reload();
        return;
    }

    if (type === 0x10) { // LOG
        const severity = chunk[5];
        const msgLen = view.getUint16(6);
        const message = new TextDecoder().decode(chunk.slice(8, 8 + msgLen));
        const styles = ['color:#aaa','color:#00d4ff;font-weight:bold','color:#ffa000;font-weight:bold','color:#f40;font-weight:bold'];
        console.log(`%c[NHTML SERVER] ${['DEBUG','INFO','WARN','ERROR'][severity] || 'LOG'}: ${message}`, styles[severity] || '');
        return;
    }

    if (type === 0x03) { // PATCH
        const opCount = view.getUint16(5);
        console.log(`%c[NHTML] 🛠️ PATCH Reçu | Ops: ${opCount}`, "color: #00ff00; font-weight: bold;");
        let offset = 7;
        for (let i = 0; i < opCount; i++) {
            const targetId = view.getUint16(offset);
            const opType = chunk[offset + 2];
            const nodeVersion = view.getUint32(offset + 3);
            console.log(`  -> Op: 0x${opType.toString(16).padStart(2,'0')} on Node ${targetId} (v${nodeVersion})`);
            offset += 7;

            const nid = window.nhtml_node_map ? window.nhtml_node_map[targetId] : (targetId === 2 ? "counter_value" : targetId);
            const el = document.querySelector(`[n-id="${nid}"]`);

            if (opType === 0x01 || opType === 0x0A || opType === 0x0B) { // Text or HTML or Append
                const len = view.getUint16(offset);
                const val = new TextDecoder().decode(chunk.slice(offset + 2, offset + 2 + len));
                offset += 2 + len;
                if (el) {
                    if (opType === 0x0B) {
                        el.insertAdjacentHTML('afterbegin', val);
                    } else {
                        el.classList.remove('nhtml-patch-glow');
                        void el.offsetWidth;
                        el.classList.add('nhtml-patch-glow');
                        if (opType === 0x0A) el.innerHTML = val; else el.textContent = val;
                    }
                    window.nhtml_last_version = nodeVersion;
                }
            } else if (opType === 0x02) { // Attr
                const kLen = chunk[offset];
                const key = new TextDecoder().decode(chunk.slice(offset + 1, offset + 1 + kLen));
                offset += 1 + kLen;
                const vLen = view.getUint16(offset);
                const val = new TextDecoder().decode(chunk.slice(offset + 2, offset + 2 + vLen));
                offset += 2 + vLen;
                if (el) el.setAttribute(key, val);
            }
        }
        return;
    }

    if (type === 0x07) { // B-TREE
        const isCompressed = chunk[5] === 0x01;
        const origLen = view.getUint32(6);
        const checksum = view.getUint32(10);
        let payload = chunk.slice(14);

        if (isCompressed) {
            if (window.fzstd) {
                try {
                    const t0 = performance.now();
                    payload = window.fzstd.decompress(payload);
                    const t1 = performance.now();
                    console.log(`%c[NHTML] 🗜️ Zstd Décompression : ${chunk.length - 14}B -> ${payload.length}B (en ${(t1-t0).toFixed(3)}ms)`, "color: #0f0; font-size: 0.7rem;");
                } catch (e) {
                    console.error("[NHTML] ❌ Erreur décompression Zstd:", e);
                    return;
                }
            } else {
                console.error("[NHTML] ❌ Erreur : Paquet Zstd reçu mais fzstd.js n'est pas chargé.");
                return;
            }
        }

        const payloadView = new DataView(payload.buffer, payload.byteOffset, payload.byteLength);
        let offset = 0;
        const nodeCount = payloadView.getUint16(offset);
        console.log(`%c[NHTML] 📦 B-TREE Snapshot | Nodes: ${nodeCount}`, "color: #00d4ff; font-weight: bold;");
        offset += 2;
        window.nhtml_node_map = window.nhtml_node_map || {};
        for (let i = 0; i < nodeCount; i++) {
            const targetId = payloadView.getUint16(offset);
            const nodeVersion = payloadView.getUint32(offset + 2);
            const valLen = payloadView.getUint16(offset + 6);
            const val = new TextDecoder().decode(payload.slice(offset + 8, offset + 8 + valLen));
            offset += 8 + valLen;
            console.log(`  -> Sync Node ${targetId}: "${val.substring(0, 20)}${val.length > 20 ? '...' : ''}" (v${nodeVersion})`);
            const nid = (targetId === 2) ? "counter_value" : targetId;
            window.nhtml_node_map[targetId] = nid;
            const el = document.querySelector(`[n-id="${nid}"]`);
            if (el) {
                el.innerText = val.trim();
                window.nhtml_last_version = Math.max(window.nhtml_last_version || 0, nodeVersion);
            }
        }
        return;
    }

    if (type === 0x09) { // RELOAD
        location.reload();
        return;
    }

    if (type === 0x10) { // LOG (Mirroring)
        const severity = chunk[5]; // [Type:1][Len:4][Sev:1]
        const len = view.getUint16(6);
        const msg = new TextDecoder().decode(chunk.slice(8, 8 + len));
        const color = (severity >= 2) ? "inherit" : "#ff8800";
        console.log(`%c[PHP] ${msg}`, `color: ${color}`);
        return;
    }

    if (polyfill) polyfill.feed_bytes(chunk);
}

function connect(wsUrl) {
    const url = new URL(wsUrl);
    url.searchParams.set("sid", window.nhtml_session_id);
    
    socket = new WebSocket(url.toString());
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
        console.log(`%c[NHTML] Connecté (SID: ${window.nhtml_session_id})`, "color: #00d4ff; font-weight: bold;");
        window.nhtml_socket = socket;
        transportMode = "WS";
        reconnectAttempts = 0;
        const hello = new Uint8Array(5 + window.nhtml_session_id.length);
        hello[0] = 0x01;
        new DataView(hello.buffer).setUint32(1, window.nhtml_last_version);
        hello.set(new TextEncoder().encode(window.nhtml_session_id), 5);
        console.log(`%c[NBPS] HELLO: ${window.nhtml_session_id} (Ver: ${window.nhtml_last_version})`, "color: #00d4ff; font-style: italic;");
        socket.send(hello);
    };

    socket.onmessage = (e) => processMessage(new Uint8Array(e.data));
    
    socket.onclose = () => {
        if (transportMode === "WS") {
            const delay = Math.min(MAX_BACKOFF, 500 * Math.pow(2, reconnectAttempts++));
            setTimeout(() => connect(wsUrl), delay);
        }
    };
}

document.addEventListener('click', (e) => {
    const target = e.target.closest('[n-click]');
    if (!target) return;
    const nid = target.getAttribute('n-id');
    const handler = target.getAttribute('n-click');
    if (nid) {
        console.log(`%c[NHTML] 🖱️ Click: ${nid} -> Handler: ${handler}`, "color: #ffaa00; font-weight: bold;");
        const id_num = (nid === 'btn_increment') ? 1 : parseInt(nid); 
        const buffer = new ArrayBuffer(5);
        const view = new DataView(buffer);
        view.setUint8(0, 0x02);
        view.setUint32(1, id_num);
        
        // Prepare JSON payload for potential WASM fallback
        const jsonPayload = JSON.stringify({ nhtml_event: 'click', node_id: nid });
        sendBinary(buffer, jsonPayload);
    }
});

// Capturer aussi les inputs pour 03-live-form et 04-style-lab
document.addEventListener('input', (e) => {
    const target = e.target;
    const nid = target.getAttribute('id'); // We use IDs in live-form / style-lab
    if (nid) {
        // Prepare JSON payload for potential WASM fallback
        const formData = {};
        formData[nid] = target.value;
        const jsonPayload = JSON.stringify({ nhtml_event: 'input', node_id: nid, form_data: formData });
        
        // Mock binary buffer (0x02 is EVENT)
        const buffer = new ArrayBuffer(5);
        new DataView(buffer).setUint8(0, 0x02);
        
        sendBinary(buffer, jsonPayload);
    }
});
