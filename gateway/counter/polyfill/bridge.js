// NHTML v0.2.1 JS Bridge
// Cette couche de 2ko s'occupe de faire le lien entre le DOM et le moteur WASM (Polyfill).

import init, { NhtmlPolyfill } from '../pkg/nhtml_polyfill.js';

let polyfill = null;
let socket = null;
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000;
// Au démarrage (F5), on repart de v0 car le DOM est vide, mais on garde le SessionID
window.nhtml_last_version = 0; 
window.nhtml_session_id = localStorage.getItem('nhtml_session_id') || "";

export async function initNhtml(wsUrl) {
    if (!polyfill) {
        await init();
        polyfill = new NhtmlPolyfill();
    }

    // On ajoute le SessionID à l'URL pour que le Gateway nous reconnaisse
    const url = new URL(wsUrl);
    if (window.nhtml_session_id) {
        url.searchParams.set("sid", window.nhtml_session_id);
    }
    connect(url.toString());
}

function connect(wsUrl) {
    console.log(`[NHTML] Tentative de connexion à ${wsUrl}...`);
    socket = new WebSocket(wsUrl);
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
        console.log("[NHTML] Connecté au Gateway.");
        reconnectAttempts = 0;

        const lastVersion = window.nhtml_last_version || 0;
        const sessionId = window.nhtml_session_id || "";

        const sessionIdBytes = new TextEncoder().encode(sessionId);
        const hello = new Uint8Array(5 + sessionIdBytes.length);
        hello[0] = 0x01;
        const view = new DataView(hello.buffer);
        view.setUint32(1, lastVersion);
        hello.set(sessionIdBytes, 5);
        
        console.log(`[NHTML] Handshake HELLO (LastVersion: ${lastVersion}, Session: ${sessionId || 'Nouvelle'})`);
        socket.send(hello);
    };

    socket.onmessage = (event) => {
        const chunk = new Uint8Array(event.data);
        console.log(`[NHTML] Message reçu: Type=0x${chunk[0].toString(16).padStart(2, '0')} (Len=${chunk.length})`);
        
        if (chunk[0] === 0x09) {
            console.log("[NHTML] 🔥 Hot Reload déclenché !");
            window.location.reload();
            return;
        }

        if (chunk[0] === 0x01) { // HELLO Serveur -> Client
            const view = new DataView(event.data);
            const keepAlive = view.getUint16(1);
            const sessionID = new TextDecoder().decode(chunk.slice(3));
            console.log(`[NHTML] Session ID reçue : ${sessionID}`);
            window.nhtml_session_id = sessionID;
            localStorage.setItem('nhtml_session_id', sessionID);
            return;
        }

        if (chunk[0] === 0x07) { // B-TREE Snapshot (Full-Path)
            const view = new DataView(event.data);
            const totalLen = view.getUint32(1);
            const compression = view.getUint8(5);
            const origLen = view.getUint32(6);
            const checksum = view.getUint32(10);
            
            console.log(`[NHTML] 📦 B-TREE Reçu (Len=${totalLen}, Compression=${compression})`);
            
            // On saute le header (1 + 4 + 1 + 4 + 4 = 14 octets)
            let offset = 14;
            const nodeCount = view.getUint16(offset);
            offset += 2;

            for (let i = 0; i < nodeCount; i++) {
                const targetId = view.getUint16(offset);
                const nodeVersion = view.getUint32(offset + 2);
                const valLen = view.getUint16(offset + 6);
                const val = new TextDecoder().decode(chunk.slice(offset + 8, offset + 8 + valLen));
                offset += 8 + valLen;

                const nid = (targetId === 2) ? "counter_value" : targetId;
                const el = document.querySelector(`[n-id="${nid}"]`);
                if (el) {
                    el.innerText = val.trim();
                    window.nhtml_last_version = Math.max(window.nhtml_last_version || 0, nodeVersion);
                    console.log(`[NHTML] B-TREE Node Sync: ${nid} (v${nodeVersion})`);
                }
            }
            return;
        }

        if (chunk[0] === 0x03) { // PATCH Binaire (NBPS v0.2.2)
            const view = new DataView(event.data);
            const opCount = view.getUint16(3);
            console.log(`[NHTML] Packet 0x03 | Ops: ${opCount} | Raw: ${Array.from(chunk).map(b => b.toString(16).padStart(2, '0')).join(' ')}`);
            let offset = 5;

            for (let i = 0; i < opCount; i++) {
                const targetId = view.getUint16(offset);
                const opType = view.getUint8(offset + 2);
                const nodeVersion = view.getUint32(offset + 3); // NodeVersion (u32)
                offset += 7;

                if (opType === 0x01) { // SET_TEXT
                    const textLen = view.getUint16(offset);
                    const text = new TextDecoder().decode(chunk.slice(offset + 2, offset + 2 + textLen));
                    offset += 2 + textLen;

                    const nid = (targetId === 2) ? "counter_value" : targetId;
                    const el = document.querySelector(`[n-id="${nid}"]`);
                    if (el) {
                        el.innerText = text.trim();
                        window.nhtml_last_version = nodeVersion;
                        localStorage.setItem('nhtml_last_version', nodeVersion);
                        console.log(`[NHTML] État mis à jour (v${nodeVersion})`);
                    }
                }
            }
            return;
        }

        if (chunk[0] === 91) { // 91 = '[' (Début de JSON - Legacy)
            try {
                const text = new TextDecoder().decode(chunk);
                const patches = JSON.parse(text);
                for (let p of patches) {
                    if (p.op === 'set_text') {
                        const el = document.querySelector(`[n-id="${p.nid}"]`);
                        if (el) el.innerText = p.value;
                    }
                }
            } catch(e) {}
            return;
        }
        polyfill.feed_bytes(chunk);
    };

    socket.onclose = () => {
        const delay = calculateBackoff();
        console.log(`[NHTML] Déconnecté. Nouvelle tentative dans ${Math.round(delay/1000)}s...`);
        setTimeout(() => connect(wsUrl), delay);
    };

    socket.onerror = (err) => {
        console.error("[NHTML] Erreur WebSocket:", err);
    };

    // Gestion des événements CLICK globaux (Délégation)
    document.addEventListener('click', (e) => {
        const target = e.target.closest('[n-click]');
        if (!target) return;
        
        const nid = target.getAttribute('n-id');
        const handler = target.getAttribute('n-click');
        
        if (nid && handler && socket.readyState === WebSocket.OPEN) {
            console.log(`[NHTML] Clic sur ${nid} (${handler})`);
            
            // Format du paquet EVENT (simplifié pour le POC)
            // [0x02] [node_id 4 bytes]
            const id_num = (nid === 'btn_increment') ? 1 : parseInt(nid); 
            const buffer = new ArrayBuffer(5);
            const view = new DataView(buffer);
            view.setUint8(0, 0x02); // PKT_EVENT
            view.setUint32(1, id_num);
            socket.send(buffer);
        }
    });
}


function calculateBackoff() {
    // Backoff exponentiel : 1s, 2s, 4s, 8s...
    let delay = Math.min(Math.pow(2, reconnectAttempts) * 1000, MAX_BACKOFF);
    // Ajout d'un JITTER (aléatoire entre 0 et 1000ms) pour éviter les tempêtes
    delay += Math.random() * 1000;
    reconnectAttempts++;
    return delay;
}

// ------------------------------------------------------------------
// Exports pour le WASM (Fonctions FFI)
// ------------------------------------------------------------------

window.nhtml_set_text = (nid, text) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.textContent = text;
};

window.nhtml_add_class = (nid, cls) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.classList.add(cls);
};

window.nhtml_remove_class = (nid, cls) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.classList.remove(cls);
};

window.nhtml_set_attr = (nid, attr, val) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.setAttribute(attr, val);
};

window.nhtml_set_style = (nid, prop, val) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.style.setProperty(prop, val);
};

window.nhtml_show = (nid) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.style.display = '';
};

window.nhtml_hide = (nid) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.style.display = 'none';
};

window.nhtml_remove = (nid) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.remove();
};

window.nhtml_replace_inner = (nid, html) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (el) el.innerHTML = html;
};

// ------------------------------------------------------------------
// LOCAL ACTIONS
// Câblage natif JS équivalent au NhtmlLocalActionRunner C++
// ------------------------------------------------------------------

window.nhtml_register_local_action = (nid, actionType, trigger, param, flags) => {
    const el = document.querySelector(`[n-id="${nid}"]`);
    if (!el) return;

    // Trigger HOVER = 0x01
    if (trigger === 0x01) {
        el.addEventListener('mouseenter', () => {
            if (actionType === 0x01) el.classList.add(param); // ADD_CLASS
        });
        
        // FLAG: Reverse on Leave
        if (flags & 0x02) {
            el.addEventListener('mouseleave', () => {
                if (actionType === 0x01) el.classList.remove(param);
            });
        }
    }
    
    // Trigger SCROLL = 0x02
    if (trigger === 0x02) {
        window.addEventListener('scroll', () => {
            const scrollY = window.scrollY;
            el.style.setProperty(param, scrollY + "px"); // SET_CSS_VAR
        });
    }

    // Trigger MOUSE_TRACK = 0x03
    if (trigger === 0x03) {
        window.addEventListener('mousemove', (e) => {
            const width = window.innerWidth;
            const xVal = (e.clientX / width) * 2.0 - 1.0;
            el.style.setProperty(param, xVal); // SET_CSS_VAR
        });
    }

    // Autres triggers (TOGGLE, etc.) à implémenter...
};
