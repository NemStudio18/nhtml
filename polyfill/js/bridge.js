// NHTML v0.2.1 JS Bridge
// Cette couche de 2ko s'occupe de faire le lien entre le DOM et le moteur WASM (Polyfill).

import init, { NhtmlPolyfill } from '../pkg/nhtml_polyfill.js';

let polyfill = null;
let socket = null;
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000; // 30 secondes max

export async function initNhtml(wsUrl) {
    if (!polyfill) {
        await init();
        polyfill = new NhtmlPolyfill();
    }

    connect(wsUrl);
}

function connect(wsUrl) {
    console.log(`[NHTML] Tentative de connexion à ${wsUrl}...`);
    socket = new WebSocket(wsUrl);
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
        console.log("[NHTML] Connecté au Gateway.");
        reconnectAttempts = 0; // Reset
    };

    socket.onmessage = (event) => {
        const chunk = new Uint8Array(event.data);
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
