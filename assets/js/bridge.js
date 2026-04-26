console.log("🛰️ NHTML v0.4.0 Bridge Loading...");

let socket = null;
let transportMode = "WS"; // "WS" ou "HTTP"
let httpFallbackUrl = "";
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000;

window.nhtml_last_version = 0; 
window.nhtml_session_id = localStorage.getItem('nhtml_session_id') || "";

/**
 * Initialise la connexion NHTML
 * @param {string} wsUrl - URL du WebSocket (ex: ws://127.0.0.1:8080)
 * @param {string} httpUrl - URL de secours HTTP (ex: /app.php)
 */
async function initNhtml(wsUrl, httpUrl = "") {
    console.log("🛰️ NHTML Initializing with WS:", wsUrl);
    httpFallbackUrl = httpUrl;
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
            pointer-events: none; opacity: 0.8;
        }
        #nhtml-hud span { color: #fff; }
    `;
    document.head.appendChild(style);
}

function injectHud() {
    if (document.getElementById('nhtml-hud')) return;
    const hud = document.createElement('div');
    hud.id = 'nhtml-hud';
    hud.innerHTML = 'NHTML v0.4.0: <span id="nhtml-hud-pkts">0</span> pkts | <span id="nhtml-hud-size">0</span> B <span id="nhtml-hud-mode">(WS)</span>';
    document.body.appendChild(hud);
    window.nhtml_stats = { pkts: 0, size: 0 };
}

async function sendBinary(data, jsonPayload = null) {
    if (transportMode === "WS" && socket && socket.readyState === 1) {
        socket.send(data);
    } else if (httpFallbackUrl) {
        // Mode HTTP Fallback
        try {
            const response = await fetch(httpFallbackUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/octet-stream', 'X-Nhtml-Session': window.nhtml_session_id },
                body: data
            });
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                const json = await response.json();
                if (json.patch) applyJsonPatches(json.patch);
            } else {
                const buffer = await response.arrayBuffer();
                if (buffer.byteLength > 0) processMessage(new Uint8Array(buffer));
            }
        } catch (e) {
            console.error("[NHTML] Erreur transport HTTP:", e);
        }
    }
}

function applyJsonPatches(patches) {
    console.log(`%c[NHTML] 🪶 PATCH JSON Reçu | Ops: ${patches.length}`, "color: #ff00ff; font-weight: bold;");
    for (const p of patches) {
        const el = document.querySelector(`[n-id="${p.nid}"]`);
        if (!el) continue;
        
        if (p.op === 'set_text') {
            el.textContent = p.val;
            triggerGlow(el);
        } else if (p.op === 'replace_inner') {
            el.innerHTML = p.val;
            triggerGlow(el);
        } else if (p.op === 'add_class') {
            p.val.split(/\s+/).filter(c => c).forEach(c => el.classList.add(c));
        } else if (p.op === 'remove_class') {
            p.val.split(/\s+/).filter(c => c).forEach(c => el.classList.remove(c));
        } else if (p.op === 'set_style') {
            el.style[p.prop] = p.val;
        } else if (p.op === 'remove') {
            el.remove();
        } else if (p.op === 'focus') {
            el.focus();
        } else if (p.op === 'scroll_to') {
            el.scrollIntoView({ behavior: 'smooth' });
        }
    }
}

function triggerGlow(el) {
    el.classList.remove('nhtml-patch-glow');
    void el.offsetWidth;
    el.classList.add('nhtml-patch-glow');
}

function processMessage(chunk) {
    if (chunk.length < 5) return;
    const view = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
    const type = chunk[0];
    
    window.nhtml_stats.pkts++;
    window.nhtml_stats.size += chunk.length;
    const hudPkts = document.getElementById('nhtml-hud-pkts');
    const hudSize = document.getElementById('nhtml-hud-size');
    const hudMode = document.getElementById('nhtml-hud-mode');
    if (hudPkts) hudPkts.innerText = window.nhtml_stats.pkts;
    if (hudSize) hudSize.innerText = (window.nhtml_stats.size > 1024) ? (window.nhtml_stats.size/1024).toFixed(1) + " KB" : window.nhtml_stats.size + " B";
    if (hudMode) hudMode.innerText = `(${transportMode})`;

    if (type === 0x09) { // PING
        const pong = new Uint8Array(5);
        pong[0] = 0x09; 
        if(socket && socket.readyState === 1) socket.send(pong);
        return;
    }

    if (type === 0x05) { // SYNC
        const serverChecksum = view.getUint32(5);
        const localChecksum = calculateLocalChecksum();
        if (serverChecksum !== localChecksum) {
            console.warn("[NHTML] 🔄 Désynchronisation !");
            sendHello();
        }
        return;
    }

    if (type === 0x7F) { // ERR
        const severity = chunk[5];
        const msgLen = view.getUint16(10);
        const message = new TextDecoder().decode(chunk.slice(12, 12 + msgLen));
        const styles = ["color:#aaa","color:#ffa500;font-weight:bold","color:#ff4500;font-weight:bold","background:#ff0000;color:#fff;padding:2px 5px"];
        console.log(`%c[NBPS ERR] ${message}`, styles[severity] || "");
        return;
    }

    if (type === 0x10) { // LOG
        const severity = chunk[5];
        const msgLen = view.getUint16(6);
        const message = new TextDecoder().decode(chunk.slice(8, 8 + msgLen));
        const styles = ['color:#aaa','color:#00d4ff;font-weight:bold','color:#ffa000;font-weight:bold','color:#f40;font-weight:bold'];
        console.log(`%c[SERVER] ${message}`, styles[severity] || '');
        return;
    }

    if (type === 0x03) { // PATCH
        const opCount = view.getUint16(5);
        console.log("🛠️  Applying", opCount, "patch operations");
        let offset = 7;
        for (let i = 0; i < opCount; i++) {
            if (offset + 9 > chunk.length) break;
            
            const targetId = view.getUint16(offset);
            const opType = chunk[offset + 2];
            const nodeVersion = view.getUint32(offset + 3);
            const dataLen = view.getUint16(offset + 7);
            offset += 9;
            
            let nid = window.nhtml_node_map ? window.nhtml_node_map[targetId] : null;
            if (!nid) nid = `Node#${targetId}`; // Fallback pour le log

            const el = document.querySelector(`[n-id="${nid}"]`) || (targetId > 0 ? document.querySelector(`[n-id="${targetId}"]`) : null);
            const data = chunk.slice(offset, offset + dataLen);
            const dataView = new DataView(data.buffer, data.byteOffset, data.byteLength);

            console.log(`🎯 Op: 0x${opType.toString(16)} | Target: ${nid} | DataLen: ${dataLen}`);

            if (opType === 0x01 || opType === 0x0A || opType === 0x0B) { // SET_TEXT, REPLACE_INNER, APPEND_HTML
                const val = new TextDecoder().decode(data.slice(2));
                if (el) {
                    if (opType === 0x0B) el.insertAdjacentHTML('beforeend', val);
                    else if (opType === 0x0A) el.innerHTML = val;
                    else {
                        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
                            el.value = val;
                        } else {
                            el.textContent = val;
                        }
                    }
                    triggerGlow(el);
                }
            } else if (opType === 0x02) { // SET_ATTR [K:str8][V:str16]
                let off = 0;
                const kLen = data[off++];
                const key = new TextDecoder().decode(data.slice(off, off + kLen)); off += kLen;
                const vLen = dataView.getUint16(off); off += 2;
                const val = new TextDecoder().decode(data.slice(off, off + vLen));
                if (el) el.setAttribute(key, val);
            } else if (opType === 0x03) { // DEL_ATTR [K:str8]
                const kLen = data[0];
                const key = new TextDecoder().decode(data.slice(1, 1 + kLen));
                if (el) el.removeAttribute(key);
            } else if (opType === 0x04 || opType === 0x05) { // ADD/REMOVE_CLASS [V:str16]
                const val = new TextDecoder().decode(data.slice(2));
                if (el) {
                    val.split(/\s+/).filter(c => c).forEach(c => {
                        if (opType === 0x04) el.classList.add(c);
                        else el.classList.remove(c);
                    });
                }
            } else if (opType === 0x08) { if (el) el.remove(); }
            else if (opType === 0x0D) { if (el) el.focus(); }
            else if (opType === 0x09) { // SET_STYLE [K:str8][V:str16]
                let off = 0;
                const kLen = data[off++];
                const k = new TextDecoder().decode(data.slice(off, off + kLen)); off += kLen;
                const vLen = dataView.getUint16(off); off += 2;
                const v = new TextDecoder().decode(data.slice(off, off + vLen));
                if (el) el.style[k] = v;
            } else if (opType === 0x0C) { if (el) el.scrollIntoView({ behavior: 'smooth' }); }
            
            offset += dataLen;
            window.nhtml_last_version = nodeVersion;
        }
        return;
    }

    if (type === 0x04) { // BIND
        const nodeId = view.getUint16(5);
        const nid = readStr8(chunk, 7);
        let offset = 8 + nid.length;
        const selector = readStr8(chunk, offset); offset += 1 + selector.length;
        const listenMask = chunk[offset++];
        const behaviorFlags = chunk[offset++];
        const debounce = chunk[offset++];
        const handler = readStr8(chunk, offset); offset += 1 + handler.length;
        
        console.log(`%c[NHTML] 🔗 BINDING Node ${nodeId} (NID: ${nid}) | Handler: ${handler} | Mask: 0x${listenMask.toString(16)}`, "color: #ffaa00");

        const el = document.querySelector(`[n-id="${nid}"]`);
        if (el) {
            if (!window.nhtml_reverse_map) window.nhtml_reverse_map = {};
            if (!window.nhtml_node_map) window.nhtml_node_map = {};
            window.nhtml_reverse_map[nid] = nodeId;
            window.nhtml_node_map[nodeId] = nid;
            el.dataset.nhtmlHandler = handler;
        }
        return;
    }

    if (type === 0x07) { // B-TREE
        const isCompressed = chunk[5] === 0x01;
        let payload = chunk.slice(14);
        if (isCompressed && window.fzstd) payload = window.fzstd.decompress(payload);

        const pView = new DataView(payload.buffer, payload.byteOffset, payload.byteLength);
        let offset = 2;
        const nodeCount = pView.getUint16(0);
        window.nhtml_node_map = {}; window.nhtml_reverse_map = {};
        for (let i = 0; i < nodeCount; i++) {
            const targetId = pView.getUint16(offset);
            const nodeVersion = pView.getUint32(offset + 2);
            const tag = readStr8(payload, offset + 6); offset += 7 + tag.length;
            const val = readStr16(pView, payload, offset); offset += 2 + val.length;
            window.nhtml_node_map[targetId] = tag;
            if (!window.nhtml_reverse_map) window.nhtml_reverse_map = {};
            window.nhtml_reverse_map[tag] = targetId;
            const el = document.querySelector(`[n-id="${tag}"]`);
            if (el) { 
                el.innerHTML = val; 
                window.nhtml_last_version = Math.max(window.nhtml_last_version || 0, nodeVersion); 
            }
        }
        console.log(`%c[NHTML] 🌳 B-TREE Applied (${nodeCount} nodes)`, "color: #00ff00; font-weight: bold;");
        return;
    }
}

function readStr8(chunk, offset) {
    if (offset >= chunk.length) return "";
    const len = chunk[offset];
    if (offset + 1 + len > chunk.length) return "";
    return new TextDecoder().decode(chunk.slice(offset + 1, offset + 1 + len));
}
function readStr16(view, chunk, offset) {
    if (offset + 2 > chunk.length) return "";
    const len = view.getUint16(offset);
    if (offset + 2 + len > chunk.length) return "";
    return new TextDecoder().decode(chunk.slice(offset + 2, offset + 2 + len));
}

function calculateLocalChecksum() {
    let hash = 0;
    if (!window.nhtml_node_map) return 0;
    const encoder = new TextEncoder();
    for (const id in window.nhtml_node_map) {
        const tag = window.nhtml_node_map[id];
        const el = document.querySelector(`[n-id="${tag}"]`);
        if (el) {
            const val = el.innerText || "";
            const bytes = encoder.encode(val);
            hash = (hash + parseInt(id) + bytes.length) >>> 0;
        }
    }
    return hash;
}

function connect(wsUrl) {
    try {
        const url = new URL(wsUrl);
        url.searchParams.set("sid", window.nhtml_session_id);
        console.log(`%c[NHTML] 🔌 Connecting to ${url.toString()}...`, "color: #00d4ff");
        
        socket = new WebSocket(url.toString());
        socket.binaryType = "arraybuffer";

        socket.onopen = () => { 
            console.log("%c[NHTML] ✅ WebSocket Connected", "color: #00ff00; font-weight: bold;");
            transportMode = "WS"; 
            reconnectAttempts = 0; 
            sendHello(); 
        };

        socket.onmessage = (e) => {
            const data = new Uint8Array(e.data);
            console.log(`%c[NHTML] 📥 INCOMING [Type: 0x${data[0].toString(16).toUpperCase()}] (${data.length} bytes)`, "color: #aaa");
            processMessage(data);
        };

        socket.onclose = (e) => {
            console.warn(`%c[NHTML] ❌ WebSocket Closed (Code: ${e.code})`, "color: #ff4500");
            const delay = Math.min(MAX_BACKOFF, 500 * Math.pow(2, reconnectAttempts++));
            setTimeout(() => connect(wsUrl), delay);
        };

        socket.onerror = (err) => { 
            console.error("[NHTML] ⚠️ WebSocket Error", err);
            transportMode = "HTTP"; 
        };
    } catch(e) { transportMode = "HTTP"; }
}

function sendHello() {
    if(!socket || socket.readyState !== 1) return;
    console.log("%c[NHTML] 🛰️ Sending HELLO...", "color: #00d4ff");
    
    const sidBytes = new TextEncoder().encode(window.nhtml_session_id);
    const hello = new Uint8Array(5 + sidBytes.length);
    hello[0] = 0x01;
    new DataView(hello.buffer).setUint32(1, sidBytes.length);
    hello.set(sidBytes, 5);
    
    socket.send(hello);
}

document.addEventListener('click', (e) => {
    const target = e.target.closest('[n-click]');
    if (!target) return;
    
    const nid = target.getAttribute('n-id') || "";
    const handlerAttr = target.getAttribute('n-click') || "";
    const id_num = window.nhtml_reverse_map ? (window.nhtml_reverse_map[nid] || 0) : 0;
    
    // Collecter les données de formulaire
    const formData = {};
    document.querySelectorAll('[n-id]').forEach(el => {
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
            formData[el.getAttribute('n-id')] = el.value;
        }
    });
    const jsonStr = JSON.stringify(formData);
    const jsonBytes = new TextEncoder().encode(jsonStr);
    const handlerBytes = new TextEncoder().encode(handlerAttr);
    
    // Format: [Type][NodeID:4][HandlerLen:1][Handler:N][PayloadLen:2][Payload:M]
    const buffer = new Uint8Array(1 + 4 + 1 + handlerBytes.length + 2 + jsonBytes.length);
    const view = new DataView(buffer.buffer);
    
    buffer[0] = 0x02; // EVENT
    view.setUint32(1, id_num);
    buffer[5] = handlerBytes.length;
    buffer.set(handlerBytes, 6);
    
    const payloadOffset = 6 + handlerBytes.length;
    view.setUint16(payloadOffset, jsonBytes.length);
    buffer.set(jsonBytes, payloadOffset + 2);
    
    sendBinary(buffer);
});

document.addEventListener('input', (e) => {
    const target = e.target;
    const nid = target.getAttribute('n-id');
    if (!nid) return;
    
    const handlerAttr = target.getAttribute('n-input') || target.getAttribute('n-change') || "";
    if (!handlerAttr) return; // Ne pas envoyer si aucun handler défini

    const id_num = window.nhtml_reverse_map ? (window.nhtml_reverse_map[nid] || 0) : 0;
    
    // Toujours envoyer un objet JSON pour la cohérence
    const formData = {};
    formData[nid] = target.value;
    const jsonStr = JSON.stringify(formData);
    const jsonBytes = new TextEncoder().encode(jsonStr);
    const handlerBytes = new TextEncoder().encode(handlerAttr);
    
    const buffer = new Uint8Array(1 + 4 + 1 + handlerBytes.length + 2 + jsonBytes.length);
    const view = new DataView(buffer.buffer);
    
    buffer[0] = 0x02;
    view.setUint32(1, id_num);
    buffer[5] = handlerBytes.length;
    buffer.set(handlerBytes, 6);
    
    const payloadOffset = 6 + handlerBytes.length;
    view.setUint16(payloadOffset, jsonBytes.length);
    buffer.set(jsonBytes, payloadOffset + 2);
    
    sendBinary(buffer);
});

document.addEventListener('keydown', (e) => {
    const target = e.target;
    const nid = target.getAttribute('n-id');
    if (!nid) return;
    
    const handlerAttr = target.getAttribute('n-keydown');
    if (!handlerAttr) return; // Ne pas envoyer si aucun handler défini

    const id_num = window.nhtml_reverse_map ? (window.nhtml_reverse_map[nid] || 0) : 0;
    
    // Toujours envoyer un objet JSON pour la cohérence
    const formData = {};
    formData[nid] = target.value;
    formData['event_key'] = e.key; // Inclure la touche pressée
    
    const jsonStr = JSON.stringify(formData);
    const jsonBytes = new TextEncoder().encode(jsonStr);
    const handlerBytes = new TextEncoder().encode(handlerAttr);
    
    const buffer = new Uint8Array(1 + 4 + 1 + handlerBytes.length + 2 + jsonBytes.length);
    const view = new DataView(buffer.buffer);
    
    buffer[0] = 0x02; // EVENT
    view.setUint32(1, id_num);
    buffer[5] = handlerBytes.length;
    buffer.set(handlerBytes, 6);
    
    const payloadOffset = 6 + handlerBytes.length;
    view.setUint16(payloadOffset, jsonBytes.length);
    buffer.set(jsonBytes, payloadOffset + 2);
    
    sendBinary(buffer);
});

// --- Auto-Initialization ---
window.addEventListener('DOMContentLoaded', () => {
    const host   = window.location.hostname || "127.0.0.1";
    const port   = 8080;
    const path   = window.location.pathname.substring(1);
    
    // Auto-init sur le port 8080 (Gateway Rust)
    initNhtml(`ws://${host}:${port}/ws?sid=AUTO&path=${path}`);
});
