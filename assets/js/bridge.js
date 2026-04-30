if (window.nhtml_initialized) {
    console.warn("🛰️ NHTML Bridge already initialized.");
} else {
window.nhtml_initialized = true;
console.log("🛰️ NHTML v0.7.1 Bridge Loading...");

let socket = null;
window.nhtml_transport_mode = "INIT";
let httpFallbackUrl = "";
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000;
const bridgeScript = document.currentScript || Array.from(document.scripts).find(s => s.src.includes('bridge.js'));
const bridgeUrl = bridgeScript ? new URL(bridgeScript.src, window.location.href) : null;
const bridgeDir = bridgeUrl ? bridgeUrl.pathname.replace(/\/[^\/]*$/, '/') : '/assets/js/';
const projectRoot = bridgeDir.endsWith('assets/js/') ? bridgeDir.slice(0, -10) : '/';

window.nhtml_stats = { pkts: 0, size: 0 };
window.nhtml_node_map = {};
window.nhtml_reverse_map = {};
window.nhtml_el_cache = new Map(); // Cache des éléments pour performance
window.nhtml_session_id = sessionStorage.getItem('nhtml_session_id') || "";
window.nhtml_seq_id = 0;
window.nhtml_session_secret = null;
window.nhtml_crypto_key = null;
window.nhtml_last_ping = 0;

async function initNhtml(wsUrl, httpUrl = "") {
    if (window.location.search.includes('wasm=1')) {
        console.log("🛰️ NHTML Force WASM mode detected.");
        injectStyles();
        injectHud();
        switchToWasm();
        return;
    }

    // Protocol fix: if we are on HTTPS, we MUST use WSS for the primary attempt
    if (window.location.protocol === "https:" && wsUrl.startsWith("ws:")) {
        wsUrl = wsUrl.replace("ws:", "wss:");
    }
    
    console.log("🛰️ NHTML Initializing with WS:", wsUrl);
    httpFallbackUrl = httpUrl;
    if (!window.nhtml_session_id || window.nhtml_session_id.length < 5) {
        window.nhtml_session_id = crypto.randomUUID().replace(/-/g, '');
        sessionStorage.setItem('nhtml_session_id', window.nhtml_session_id);
    }
    injectStyles();
    injectHud();

    // Aggressive Fallback: If we are on GitHub Pages or similar static host, 
    // we might want to skip the WS wait entirely or fail very fast.
    const isStaticHost = window.location.hostname.endsWith('.github.io');
    if (isStaticHost) {
        console.log("🛰️ NHTML Static Host detected. Preparing WASM Fallback...");
        // Still try connect, but with a very short timeout
        connect(wsUrl, 1500); 
    } else {
        connect(wsUrl);
    }
}

function injectStyles() {
    if (document.getElementById('nhtml-styles')) return;
    const style = document.createElement('style');
    style.id = 'nhtml-styles';
    style.innerHTML = `
        .nhtml-patch-glow { animation: nhtml-glow 0.6s ease-out; }
        #nhtml-hud {
            position: fixed; bottom: 15px; right: 15px;
            background: rgba(10, 10, 10, 0.95);
            backdrop-filter: blur(10px);
            color: #fff;
            padding: 12px 16px;
            border-radius: 14px;
            font-family: 'Inter', system-ui, -apple-system, sans-serif;
            font-size: 11px;
            z-index: 100000;
            border: 1px solid rgba(255, 255, 255, 0.1);
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            min-width: 220px;
            cursor: pointer;
            pointer-events: auto;
        }
        #nhtml-hud:hover { border-color: rgba(255, 255, 255, 0.3); transform: translateY(-2px); }
        #nhtml-hud.minimized { width: 40px; min-width: 40px; height: 40px; border-radius: 50%; padding: 0; display: flex; align-items: center; justify-content: center; }
        #nhtml-hud.minimized .hud-content { display: none; }
        #nhtml-hud.minimized::after { content: 'N'; font-weight: bold; font-size: 14px; color: #ff007f; }
        
        .hud-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 8px; }
        .hud-title { font-weight: 800; color: #ff007f; letter-spacing: 0.5px; text-transform: uppercase; font-size: 10px; }
        .hud-row { display: flex; justify-content: space-between; margin-bottom: 4px; color: rgba(255,255,255,0.6); }
        .hud-val { color: #fff; font-family: monospace; font-weight: bold; }
        .hud-status { display: flex; align-items: center; gap: 6px; font-size: 10px; margin-top: 8px; }
        .status-dot { width: 6px; height: 6px; border-radius: 50%; background: #555; }
        .status-online { background: #22c55e; box-shadow: 0 0 8px #22c55e; }
        .status-connecting { background: #eab308; }
        .status-offline { background: #ef4444; }
    `;
    document.head.appendChild(style);
}

function injectHud() {
    if (document.getElementById('nhtml-hud')) return;
    const hud = document.createElement('div');
    hud.id = 'nhtml-hud';
    hud.onclick = () => hud.classList.toggle('minimized');
    hud.innerHTML = `
        <div class="hud-content">
            <div class="hud-header">
                <span class="hud-title">NHTML v0.7.1</span>
                <span style="opacity: 0.3; font-size: 9px;">GLOBAL CONNECT</span>
            </div>
            <div class="hud-row"><span>Transport</span><span id="nhtml-hud-mode" class="hud-val">WS</span></div>
            <div class="hud-row"><span>Packets</span><span id="nhtml-hud-pkts" class="hud-val">0</span></div>
            <div class="hud-row"><span>Traffic</span><span id="nhtml-hud-size" class="hud-val">0 K</span></div>
            <div class="hud-row"><span>Latency</span><span id="nhtml-hud-latency" class="hud-val">-- ms</span></div>
            <div class="hud-status">
                <div id="nhtml-hud-dot" class="status-dot"></div>
                <span id="nhtml-hud-status-text">Disconnected</span>
            </div>
        </div>
    `;
    document.body.appendChild(hud);
}

function triggerGlow(el) {
    if (!el) return;
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('debug_glow') !== '1') return;
    
    el.classList.remove('nhtml-patch-glow');
    void el.offsetWidth;
    el.classList.add('nhtml-patch-glow');
}

function getCachedEl(nid, targetId) {
    if (window.nhtml_el_cache.has(nid)) return window.nhtml_el_cache.get(nid);
    let el = document.querySelector(`[n-id="${nid}"]`);
    if (!el && targetId > 0) el = document.querySelector(`[n-id="${targetId}"]`);
    if (el) window.nhtml_el_cache.set(nid, el);
    return el;
}

function readStr8(chunk, offset) {
    const len = chunk[offset];
    return new TextDecoder().decode(chunk.slice(offset + 1, offset + 1 + len));
}

function readStr16(view, chunk, offset) {
    const len = view.getUint16(offset);
    return new TextDecoder().decode(chunk.slice(offset + 2, offset + 2 + len));
}

// --- Security: Safe HTML Injection ---
function safeHTML(html) {
    if (window.DOMPurify) {
        return window.DOMPurify.sanitize(html, {
            ADD_TAGS: ["n-id", "n-click", "n-input", "n-submit", "n-live", "n-model", "n-text", "n-bind"],
            ADD_ATTR: ["n-id", "n-click", "n-input", "n-submit", "n-live", "n-model", "n-text", "n-bind"]
        });
    }
    
    // Fallback basic sanitizer (Aggressive)
    const template = document.createElement('template');
    template.innerHTML = html;
    
    const scripts = template.content.querySelectorAll('script, iframe, object, embed, base');
    scripts.forEach(s => s.remove());
    
    const all = template.content.querySelectorAll('*');
    all.forEach(el => {
        const attrs = el.attributes;
        for (let i = attrs.length - 1; i >= 0; i--) {
            const attr = attrs[i];
            const attrName = attr.name.toLowerCase();
            const attrVal = attr.value.toLowerCase().trim();
            
            // Rejet des event handlers et javascript:
            if (attrName.startsWith('on') || attrVal.startsWith('javascript:') || attrVal.startsWith('data:text/html')) {
                el.removeAttribute(attr.name);
            }
            // Protection contre les imports CSS malveillants
            if (attrName === 'style' && (attrVal.includes('url(') || attrVal.includes('expression('))) {
                el.removeAttribute(attr.name);
            }
        }
    });
    
    const div = document.createElement('div');
    div.appendChild(template.content);
    return div.innerHTML;
}

// Auto-load DOMPurify in dev mode if missing
if (!window.DOMPurify && window.location.hostname === '127.0.0.1') {
    const s = document.createElement('script');
    s.src = "https://cdnjs.cloudflare.com/ajax/libs/dompurify/3.0.8/purify.min.js";
    document.head.appendChild(s);
}

// ── Local Actions Architecture ──
window.nhtml_global_actions = { scroll: [], mousemove: [] };
window.nhtml_global_listeners_attached = false;

function nhtml_apply_effect(el, actionType, param, reverse = false) {
    if (!el) return;
    switch (actionType) {
        case 0x01: // LA_ADD_CLASS
            if (reverse) el.classList.remove(param);
            else el.classList.add(param);
            break;
        case 0x02: // LA_REMOVE_CLASS
            if (reverse) el.classList.add(param);
            else el.classList.remove(param);
            break;
        case 0x03: // LA_TOGGLE_CLASS
            el.classList.toggle(param);
            break;
        case 0x04: // LA_SET_STYLE
            const sep = param.indexOf(':');
            if (sep !== -1) {
                const prop = param.slice(0, sep).trim();
                if (!reverse) {
                    const val = param.slice(sep + 1).trim();
                    el.style.setProperty(prop, val);
                } else {
                    el.style.removeProperty(prop);
                }
            }
            break;
        case 0x05: // LA_CSS_VAR_SCROLL
            // Handled dynamically in runner
            break;
        case 0x09: // LA_TOGGLE_TARGET
            const sep2 = param.indexOf(':');
            if (sep2 !== -1) {
                const targetNid = param.slice(0, sep2);
                const cls = param.slice(sep2 + 1);
                const target = document.querySelector(`[n-id="${targetNid}"]`);
                if (target) target.classList.toggle(cls);
            }
            break;
        case 0x0A: // LA_DRAG_ENABLE
            console.warn("[NHTML] LA_DRAG_ENABLE not yet implemented.");
            break;
    }
}

function nhtml_run_scroll_actions(entry) {
    const scrollY = window.scrollY;
    const maxScroll = document.body.scrollHeight - window.innerHeight;
    const ratio = maxScroll > 0 ? scrollY / maxScroll : 0;
    
    const threshold = entry.la.threshold_x10 / 100; // x10 to decimal
    
    if (entry.la.actionType === 0x05) { // CSS_VAR_SCROLL
        entry.el.style.setProperty(`--${entry.la.param}`, ratio);
    } else if (entry.la.actionType === 0x01 || entry.la.actionType === 0x02) {
        const isPast = ratio >= threshold;
        if (isPast && !entry._triggered) {
            nhtml_apply_effect(entry.el, entry.la.actionType, entry.la.param, false);
            entry._triggered = true;
            if (entry.la.flags & 0x01) entry._done = true; // LA_FLAG_ONCE
        } else if (!isPast && entry._triggered && (entry.la.flags & 0x02)) {
            nhtml_apply_effect(entry.el, entry.la.actionType, entry.la.param, true);
            entry._triggered = false;
        }
    }
}

let nhtml_rect_cache = new WeakMap();
let nhtml_resize_observer = null;

function nhtml_get_cached_rect(el) {
    if (!nhtml_resize_observer) {
        nhtml_resize_observer = new ResizeObserver(entries => {
            for (let e of entries) {
                const r = e.target.getBoundingClientRect();
                nhtml_rect_cache.set(e.target, {
                    absLeft: r.left + window.scrollX,
                    absTop: r.top + window.scrollY,
                    width: r.width,
                    height: r.height
                });
            }
        });
    }
    
    let cached = nhtml_rect_cache.get(el);
    if (!cached) {
        const r = el.getBoundingClientRect();
        cached = {
            absLeft: r.left + window.scrollX,
            absTop: r.top + window.scrollY,
            width: r.width,
            height: r.height
        };
        nhtml_rect_cache.set(el, cached);
        nhtml_resize_observer.observe(el);
    }
    
    return {
        left: cached.absLeft - window.scrollX,
        top: cached.absTop - window.scrollY,
        width: cached.width,
        height: cached.height
    };
}

function nhtml_run_mousemove_actions(entry, e) {
    if (entry._done) return;
    let x = e.clientX, y = e.clientY;
    if (entry.la.flags & 0x04) { // SCOPE_SELF
        const rect = nhtml_get_cached_rect(entry.el);
        x -= rect.left; y -= rect.top;
    }
    
    if (entry.la.actionType === 0x06) entry.el.style.setProperty(`--${entry.la.param}`, x);
    else if (entry.la.actionType === 0x07) entry.el.style.setProperty(`--${entry.la.param}`, y);
    else if (entry.la.actionType === 0x08) {
        // Distance
        const rect = nhtml_get_cached_rect(entry.el);
        const cx = rect.left + rect.width/2;
        const cy = rect.top + rect.height/2;
        const dist = Math.sqrt(Math.pow(e.clientX - cx, 2) + Math.pow(e.clientY - cy, 2));
        entry.el.style.setProperty(`--${entry.la.param}`, Math.round(dist) + 'px');
    }
    
    if (entry.la.flags & 0x01) entry._done = true;
}

function nhtml_attach_global_listeners() {
    if (window.nhtml_global_listeners_attached) return;
    window.nhtml_global_listeners_attached = true;

    window.addEventListener('scroll', () => {
        window.nhtml_global_actions.scroll = window.nhtml_global_actions.scroll.filter(entry => {
            nhtml_run_scroll_actions(entry);
            return !entry._done;
        });
    }, { passive: true });

    window.addEventListener('mousemove', (e) => {
        window.nhtml_global_actions.mousemove = window.nhtml_global_actions.mousemove.filter(entry => {
            nhtml_run_mousemove_actions(entry, e);
            return !entry._done;
        });
    }, { passive: true });
}

function nhtml_register_action(nodeId, el, la) {
    if (!el) return;
    
    // Global Listeners
    if (la.triggerType === 0x02 || la.triggerType === 0x03) { // SCROLL
        window.nhtml_global_actions.scroll.push({nodeId, el, la, _triggered: false, _done: false});
    } else if (la.triggerType === 0x04) { // MOUSEMOVE_WIN
        window.nhtml_global_actions.mousemove.push({nodeId, el, la, _done: false});
    } 
    // Direct Listeners
    else if (la.triggerType === 0x05) { // MOUSEMOVE_SELF
        el.addEventListener('mousemove', (e) => {
            nhtml_run_mousemove_actions({nodeId, el, la, _done: false}, e);
        }, { passive: true });
    }
    else if (la.triggerType === 0x01) { // HOVER
        const enterFn = () => nhtml_apply_effect(el, la.actionType, la.param, false);
        const leaveFn = () => {
            if (la.flags & 0x02) nhtml_apply_effect(el, la.actionType, la.param, true);
        };
        el.addEventListener('mouseenter', enterFn);
        el.addEventListener('mouseleave', leaveFn);
    } else if (la.triggerType === 0x06) { // FOCUS
        const fFn = () => nhtml_apply_effect(el, la.actionType, la.param, false);
        const bFn = () => {
            if (la.flags & 0x02) nhtml_apply_effect(el, la.actionType, la.param, true);
        };
        el.addEventListener('focus', fFn);
        el.addEventListener('blur', bFn);
    } else if (la.triggerType === 0x07) { // CLICK_LOCAL
        const cFn = () => {
            nhtml_apply_effect(el, la.actionType, la.param, false);
            if (la.flags & 0x01) el.removeEventListener('click', cFn);
        };
        el.addEventListener('click', cFn);
    }
}

function nhtml_init_local_actions(nodeId, binding) {
    const el = document.querySelector(`[n-id="${binding.nid}"]`) || getCachedEl(binding.nid, nodeId);
    if (!el) return;
    
    // Purge old actions for this node to avoid duplicates on re-BIND
    window.nhtml_global_actions.scroll = window.nhtml_global_actions.scroll.filter(x => x.nodeId !== nodeId);
    window.nhtml_global_actions.mousemove = window.nhtml_global_actions.mousemove.filter(x => x.nodeId !== nodeId);
    
    nhtml_attach_global_listeners();
    
    for (const la of binding.localActions) {
        nhtml_register_action(nodeId, el, la);
    }
}

function processMessage(msg) {
    try {
        const chunk = new Uint8Array(msg);
        const view = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
        const type = chunk[0];

        window.nhtml_stats.pkts++;
        window.nhtml_stats.size += chunk.length;
        const hudPkts = document.getElementById('nhtml-hud-pkts');
        if (hudPkts) hudPkts.innerText = window.nhtml_stats.pkts;
        const hudSize = document.getElementById('nhtml-hud-size');
        if (hudSize) hudSize.innerText = (window.nhtml_stats.size / 1024).toFixed(1) + 'K';

        if (type === 0x01) { // HELLO RESPONSE
            if (window.nhtml_hello_timeout) clearTimeout(window.nhtml_hello_timeout);
            window.nhtml_transport_mode = "WS_READY";
            console.log("🛰️ NHTML Gateway Handshake Complete (Binary Mode).");
            
            // Extract Secret (v0.5.0 Security)
            const sidLen = chunk[6];
            const secretOffset = 7 + sidLen;
            if (chunk.length >= secretOffset + 32) {
                window.nhtml_session_secret = chunk.slice(secretOffset, secretOffset + 32);
                
                // Extract LastSeq (v0.5.0)
                const seqOffset = secretOffset + 32;
                if (chunk.length >= seqOffset + 4) {
                    const seqView = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
                    window.nhtml_seq_id = seqView.getUint32(seqOffset);
                    console.log(`🛰️ NHTML Sequence synchronized: starting at ${window.nhtml_seq_id}`);
                }

                console.log("🛰️ NHTML Security Key received. Deriving HMAC...");
                crypto.subtle.importKey(
                    "raw", window.nhtml_session_secret,
                    { name: "HMAC", hash: { name: "SHA-256" } },
                    false, ["sign"]
                ).then(key => {
                    window.nhtml_crypto_key = key;
                    console.log("🛰️ NHTML HMAC Engine ready.");
                });
            }

            const dot = document.getElementById('nhtml-hud-dot');
            const statusText = document.getElementById('nhtml-hud-status-text');
            if (dot) {
                dot.className = 'status-dot status-online';
                if (statusText) statusText.innerText = 'Connected (Secure)';
            }
            return;
        }

        if (type === 0x09) { // PONG (from server)
            const now = Date.now();
            const latency = now - window.nhtml_last_ping;
            const hudLatency = document.getElementById('nhtml-hud-latency');
            if (hudLatency) hudLatency.innerText = latency + ' ms';
            return;
        }

        if (type === 0x10) { // LOG
            const severity = chunk[5];
            const msgLen = view.getUint16(6);
            const message = new TextDecoder().decode(chunk.slice(8, 8 + msgLen));
            
            if (severity === 0x11 && message === "RELOAD") {
                console.log("🔄 Hot Reload: Reloading page...");
                window.location.reload();
                return;
            }

            const styles = ['color:#aaa','color:#00d4ff;font-weight:bold','color:#ffa000;font-weight:bold','color:#f40;font-weight:bold'];
            console.log(`%c[SERVER] ${message}`, styles[severity] || '');
            return;
        }

        if (type === 0x03) { // PATCH
            const opCount = view.getUint16(5);
            let offset = 7;
            for (let i = 0; i < opCount; i++) {
                const targetId = view.getUint16(offset);
                const opType = chunk[offset + 2];
                const nodeVersion = view.getUint32(offset + 3);
                const dataLen = view.getUint16(offset + 7);
                offset += 9;

                const nid = window.nhtml_node_map[targetId];
                const el = getCachedEl(nid, targetId);
                const data = chunk.slice(offset, offset + dataLen);
                const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

                if (el) {
                    let opName = "UNKNOWN";
                    switch (opType) {
                        case 0x01: // SET_TEXT
                            opName = "SET_TEXT";
                            const txt = new TextDecoder().decode(data.slice(2));
                            if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') el.value = txt;
                            else el.textContent = txt;
                            break;
                        case 0x02: // SET_ATTR
                            opName = "SET_ATTR";
                            const akLen = data[0];
                            const ak = new TextDecoder().decode(data.slice(1, 1 + akLen));
                            const avLen = dv.getUint16(1 + akLen);
                            const av = new TextDecoder().decode(data.slice(3 + akLen, 3 + akLen + avLen));
                            el.setAttribute(ak, av);
                            break;
                        case 0x03: // DEL_ATTR
                            opName = "DEL_ATTR";
                            el.removeAttribute(new TextDecoder().decode(data.slice(1, 1 + data[0])));
                            break;
                        case 0x04: // ADD_CLASS
                            opName = "ADD_CLASS";
                            el.classList.add(new TextDecoder().decode(data.slice(2)));
                            break;
                        case 0x05: // REM_CLASS
                            opName = "REM_CLASS";
                            el.classList.remove(new TextDecoder().decode(data.slice(2)));
                            break;
                        case 0x06: // INSERT_BEFORE
                            opName = "INSERT_BEFORE";
                            el.insertAdjacentHTML('beforebegin', safeHTML(new TextDecoder().decode(data.slice(2))));
                            break;
                        case 0x07: // INSERT_AFTER
                            opName = "INSERT_AFTER";
                            el.insertAdjacentHTML('afterend', safeHTML(new TextDecoder().decode(data.slice(2))));
                            break;
                        case 0x08: // REMOVE
                            opName = "REMOVE";
                            el.remove();
                            window.nhtml_el_cache.delete(nid);
                            break;
                        case 0x09: // SET_STYLE
                            opName = "SET_STYLE";
                            const skLen = data[0];
                            const sk = new TextDecoder().decode(data.slice(1, 1 + skLen));
                            const svLen = dv.getUint16(1 + skLen);
                            const sv = new TextDecoder().decode(data.slice(3 + skLen, 3 + skLen + svLen));
                            el.style[sk] = sv;
                            break;
                        case 0x0A: // REPLACE_INNER
                            opName = "REPLACE_INNER";
                            el.innerHTML = safeHTML(new TextDecoder().decode(data.slice(2)));
                            break;
                        case 0x0B: // APPEND_HTML
                            opName = "APPEND_HTML";
                            el.insertAdjacentHTML('beforeend', safeHTML(new TextDecoder().decode(data.slice(2))));
                            break;
                        case 0x0C: // SCROLL_TO
                            opName = "SCROLL_TO";
                            if (el.scrollHeight > el.clientHeight) {
                                el.scrollTop = el.scrollHeight;
                            } else {
                                el.scrollIntoView({ behavior: 'smooth', block: 'end' });
                            }
                            break;
                        case 0x0D: // FOCUS
                            opName = "FOCUS";
                            el.focus();
                            break;
                    }
                    console.log(`%c[UI PATCH] %c${opName} %con node %c#${nid}`, 
                        'color:#ff007f;font-weight:bold', 'color:#fff', 'color:#aaa', 'color:#00d4ff');
                    if (opType < 0x0C) triggerGlow(el);

                    // Re-scan IDs if this was a content replacement
                    if (opType === 0x0A || opType === 0x0B) {
                        el.querySelectorAll('[n-id]').forEach(subEl => {
                            const subNid = subEl.getAttribute('n-id');
                            const subIdNum = parseInt(subNid);
                            if (!isNaN(subIdNum)) {
                                window.nhtml_node_map[subIdNum] = subNid;
                                window.nhtml_reverse_map[subNid] = subIdNum;
                            }
                        });
                    }
                }
                offset += dataLen;
            }
            return;
        }

        if (type === 0x04) { // BIND
            const nodeId = view.getUint16(5);
            let bOff = 7;
            const nidLen = chunk[bOff++];
            const nid = new TextDecoder().decode(chunk.slice(bOff, bOff + nidLen));
            bOff += nidLen;
            
            const selLen = chunk[bOff++];
            const selector = new TextDecoder().decode(chunk.slice(bOff, bOff + selLen));
            bOff += selLen;
            
            const listenMask = chunk[bOff++];
            const behaviorFlags = chunk[bOff++];
            const debounce = chunk[bOff++] * 100; // debounce_100ms
            
            const handlerLen = chunk[bOff++];
            const handler = new TextDecoder().decode(chunk.slice(bOff, bOff + handlerLen));
            bOff += handlerLen;
            
            const modelLen = chunk[bOff++];
            const nModel = new TextDecoder().decode(chunk.slice(bOff, bOff + modelLen));
            bOff += modelLen;
            
            const textLen = chunk[bOff++];
            const nText = new TextDecoder().decode(chunk.slice(bOff, bOff + textLen));
            bOff += textLen;
            
            const laCount = chunk[bOff++];
            const localActions = [];
            for (let i = 0; i < laCount; i++) {
                const actionType = chunk[bOff++];
                const triggerType = chunk[bOff++];
                
                const paramLen = chunk[bOff++];
                const param = new TextDecoder().decode(chunk.slice(bOff, bOff + paramLen));
                bOff += paramLen;
                
                const flags = chunk[bOff++];
                const threshold_x10 = chunk[bOff++];
                localActions.push({actionType, triggerType, param, flags, threshold_x10});
            }

            window.nhtml_reverse_map[nid] = nodeId;
            window.nhtml_node_map[nodeId] = nid;
            window.nhtml_bindings = window.nhtml_bindings || {};
            window.nhtml_bindings[nodeId] = {
                nid, selector, listenMask, behaviorFlags, debounce, handler, nModel, nText, localActions
            };
            
            nhtml_init_local_actions(nodeId, window.nhtml_bindings[nodeId]);
            return;
        }

        if (type === 0x07) { // B-TREE
            const isCompressed = chunk[5] === 0x01;
            const origLen = view.getUint32(6);
            const checksum = view.getUint32(10);
            let btreeData = chunk.slice(14);

            if (isCompressed) {
                if (window.fzstd) {
                    btreeData = window.fzstd.decompress(btreeData);
                    if (btreeData.length !== origLen) {
                        console.error(`[NHTML] B-TREE Integrity Error: Expected ${origLen} bytes, got ${btreeData.length}`);
                        return;
                    }
                } else {
                    console.error("[NHTML] B-TREE is compressed but fzstd is missing! Using raw data (will fail).");
                }
            }

            const pView = new DataView(btreeData.buffer, btreeData.byteOffset, btreeData.byteLength);
            const nodeCount = pView.getUint16(0);
            let pOff = 2;
            window.nhtml_el_cache.clear();
            
            for (let i = 0; i < nodeCount; i++) {
                if (btreeData.length < pOff + 2) break;
                const targetId = pView.getUint16(pOff);
                // Schema: [ID:2][Ver:4][TagLen:1][Tag:str][ValLen:2][Val:str]
                const tagLen = btreeData[pOff + 6];
                const tag = new TextDecoder().decode(btreeData.slice(pOff + 7, pOff + 7 + tagLen));
                pOff += 7 + tagLen;
                
                if (btreeData.length < pOff + 2) break;
                const valLen = pView.getUint16(pOff);
                const val = new TextDecoder().decode(btreeData.slice(pOff + 2, pOff + 2 + valLen));
                pOff += 2 + valLen;
                
                window.nhtml_node_map[targetId] = tag;
                window.nhtml_reverse_map[tag] = targetId;
                const el = document.querySelector(`[n-id="${tag}"]`);
                if (el) {
                    window.nhtml_el_cache.set(tag, el);
                }
            }
            console.log(`[NHTML] 🌳 B-TREE Applied (${nodeCount} nodes, Checksum: ${checksum.toString(16)})`);
            return;
        }
    } catch (e) {
        console.error("[NHTML] Critical error in processMessage:", e);
    }
}

function connect(wsUrl, timeoutMs = 5000) {
    if (window.nhtml_transport_mode === "WASM") return; 
    window.nhtml_transport_mode = "WS_CONNECTING";
    
    try {
        const url = new URL(wsUrl);
        url.searchParams.set("sid", window.nhtml_session_id);
        socket = new WebSocket(url.toString());
        socket.binaryType = "arraybuffer";
        
        // Timeout for initial connection
        const connTimeout = setTimeout(() => {
            if (socket.readyState !== 1) {
                console.warn("🛰️ NHTML Connection timeout. Falling back...");
                socket.close();
                switchToWasm();
            }
        }, timeoutMs);

        socket.onopen = () => { 
            clearTimeout(connTimeout);
            if (window.nhtml_transport_mode === "WASM") {
                socket.close();
                return;
            }
            
            // Start a secondary timeout for the HELLO handshake
            window.nhtml_hello_timeout = setTimeout(() => {
                if (window.nhtml_transport_mode !== "WS_READY") {
                    console.warn("🛰️ NHTML Handshake timeout. Falling back to WASM...");
                    socket.close();
                    switchToWasm();
                }
            }, 2500);

            window.nhtml_transport_mode = "WS_OPEN"; 
            reconnectAttempts = 0; 
            sendHello(); 
            
            // Latency Monitoring
            setInterval(() => {
                if (socket && socket.readyState === 1) {
                    window.nhtml_last_ping = Date.now();
                    const ping = new Uint8Array(5); ping[0] = 0x09;
                    socket.send(ping);
                }
            }, 3000);
        };
        
        socket.onmessage = (e) => processMessage(e.data);
        
        socket.onerror = (err) => {
            clearTimeout(connTimeout);
            console.warn("🛰️ NHTML WebSocket Error, checking fallback...");
            switchToWasm();
        };

        socket.onclose = () => { 
            clearTimeout(connTimeout);
            if (window.nhtml_hello_timeout) clearTimeout(window.nhtml_hello_timeout);
            
            if (window.nhtml_transport_mode.startsWith("WS")) {
                setTimeout(() => connect(wsUrl), Math.min(MAX_BACKOFF, 500 * Math.pow(2, reconnectAttempts++))); 
            } else {
                switchToWasm();
            }
        };
    } catch(e) { 
        console.error("🛰️ NHTML Connection fail:", e);
        switchToWasm();
    }
}

let nhtml_wasm_init_started = false;
async function switchToWasm() {
    if (window.nhtml_transport_mode === "WASM" || nhtml_wasm_init_started) return;
    
    // Secure Context Check
    if (window.location.protocol !== "https:" && window.location.hostname !== "localhost" && window.location.hostname !== "127.0.0.1") {
        console.error("🛰️ NHTML WASM fallback aborted: Requires HTTPS (Secure Context) in production.");
        return;
    }

    nhtml_wasm_init_started = true;
    
    window.nhtml_transport_mode = "WASM";
    console.log("🛰️ NHTML Switching to ZERO-SERVER (WASM) Mode...");
    const hudMode = document.getElementById('nhtml-hud-mode');
    if (hudMode) hudMode.innerText = "(WASM)";

    const statusBadge = document.getElementById('nhtml-status-badge');
    if (statusBadge) {
        statusBadge.innerHTML = '<div style="width: 8px; height: 8px; background: #38bdf8; border-radius: 50%; box-shadow: 0 0 10px #38bdf8;"></div> WASM Mode (Local)';
    }

    try {
        if (!window.PhpWeb) {
            const module = await import(bridgeDir + 'php-wasm/PhpWeb.mjs');
            window.PhpWeb = module.PhpWeb;
        }
        
        window.nhtml_wasm_php = new window.PhpWeb({ persist: true });
        const php = await window.nhtml_wasm_php.binary;

        // --- SAFE PERSISTENCE LAYER ---
        const MOUNT_POINT = '/persist';
        
        // Check if already mounted (to avoid EBUSY)
        let isMounted = false;
        try {
            const m = php.FS.analyzePath(MOUNT_POINT);
            if (m.exists && m.object.mount) isMounted = true;
        } catch(e) {}

        if (!isMounted) {
            try { php.FS.mkdir(MOUNT_POINT); } catch(e){}
            php.FS.mount(php.FS.filesystems.IDBFS, {}, MOUNT_POINT);
            await new Promise(resolve => php.FS.syncfs(true, resolve));
            console.log("🛰️ NHTML Persistence Layer Mounted & Synced.");
        }
        // ------------------------------
        
        const basePath = window.location.pathname.replace(/\/[^\/]*$/, '/');
        const cacheBust = "?v=" + Date.now();
        
        const resApp = await fetch(basePath + 'app.php' + cacheBust);
        if (resApp.ok) {
            let code = await resApp.text();
            code = code.replace(/file_get_contents\s*\(\s*['"]php:\/\/stdin['"]\s*\)/g, "($input ?? file_get_contents('php://stdin'))");
            php.FS.writeFile('/app.php', code);
        }

        const sdkFiles = [projectRoot + 'sdk/php/src/Nhtml.php', projectRoot + 'sdk/php/src/Patch.php'];
        for (const f of sdkFiles) {
            const res = await fetch(f + cacheBust);
            if (res.ok) {
                const sdkCode = await res.text();
                const parts = f.split('/');
                let current = '';
                for (let i = 0; i < parts.length - 1; i++) {
                    current += (current ? '/' : '') + parts[i];
                    try { php.FS.mkdir(current); } catch(e){}
                }
                php.FS.writeFile(f, sdkCode);
            }
        }

        console.log("🛰️ NHTML WASM Filesystem Ready. Hydrating...");
        wasmRunEvent(0, 'init', {});
        
    } catch (e) {
        console.error("🛰️ NHTML WASM Init Failed:", e);
    }
}

function applyJsonPatch(ops) {
    if (!Array.isArray(ops)) return;
    console.log(`%c[WASM PATCH] %cApplying ${ops.length} operations`, 'color:#ff007f;font-weight:bold', 'color:#fff');
    
    ops.forEach(op => {
        const nid = op.nid || op.id;
        const el = document.querySelector(`[n-id="${nid}"]`);
        if (!el) {
            console.warn(`🛰️ NHTML Patch: Node ${nid} not found for op ${op.op}`);
            return;
        }
        
        console.log(`  [OP] ${op.op} on ${nid}`, op);

        switch (op.op) {
            case 'set_text':
                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') el.value = op.val;
                else el.textContent = op.val;
                break;
            case 'set_html':
            case 'replace_inner': 
                el.innerHTML = safeHTML(op.val); 
                break;
            case 'add_class': el.classList.add(op.val); break;
            case 'del_class': 
            case 'remove_class': el.classList.remove(op.val); break;
            case 'set_attr': el.setAttribute(op.key || op.name, op.val); break;
            case 'del_attr': el.removeAttribute(op.key || op.name); break;
            case 'set_style': 
                const prop = op.prop || op.key;
                el.style[prop] = op.val; 
                break;
            case 'append_html': el.insertAdjacentHTML('beforeend', safeHTML(op.val)); break;
            case 'remove': el.remove(); break;
            case 'scroll_to': el.scrollIntoView({ behavior: 'smooth', block: 'end' }); break;
            case 'focus': el.focus(); break;
        }
        triggerGlow(el);
    });
}

function sendHello() {
    if(!socket || socket.readyState !== 1) return;
    const sidBytes = new TextEncoder().encode(window.nhtml_session_id);
    
    // NBPS v0.5.0 HELLO (Client->Server): [Type:1][Len:4][SIDLen:1][SID:var]
    const totalSize = 1 + 4 + 1 + sidBytes.length;
    const buf = new Uint8Array(totalSize);
    const view = new DataView(buf.buffer);
    
    buf[0] = 0x01; // Type HELLO
    view.setUint32(1, totalSize - 5); // Length after header
    buf[5] = sidBytes.length;
    buf.set(sidBytes, 6);
    
    sendBinary(buf);
}

function sendBinary(data) {
    if (socket && socket.readyState === 1) socket.send(data);
}
window.sendBinary = sendBinary;

window.nhtml_debounce_timers = {};

async function sendEvent(id_num, handler, formData) {
    const jsonBytes = new TextEncoder().encode(JSON.stringify(formData));
    const hBytes = new TextEncoder().encode(handler);
    
    // NBPS v0.5.0: [Type:1][Len:4][Seq:4][Sig:32][Payload...]
    // Payload = [NodeID:4][HLen:1][Handler:str][PLen:2][Payload:json]
    const payloadSize = 4 + 1 + hBytes.length + 2 + jsonBytes.length;
    const totalSize = 1 + 4 + 4 + 32 + payloadSize;
    
    const buf = new Uint8Array(totalSize);
    const view = new DataView(buf.buffer);
    
    window.nhtml_seq_id++;
    const currentSeq = window.nhtml_seq_id;
    
    buf[0] = 0x02; // Type EVENT
    view.setUint32(1, totalSize - 5); // Length after header
    view.setUint32(5, currentSeq); // Sequence ID
    
    // Skip 32 bytes for signature (offset 9 to 41)
    const payloadOffset = 41;
    view.setUint32(payloadOffset, id_num);
    buf[payloadOffset + 4] = hBytes.length;
    buf.set(hBytes, payloadOffset + 5);
    
    const pLenOff = payloadOffset + 5 + hBytes.length;
    view.setUint16(pLenOff, jsonBytes.length);
    buf.set(jsonBytes, pLenOff + 2);

    // Sign the packet (v0.5.0)
    if (window.nhtml_crypto_key) {
        // Data to sign: [Type][Len][Seq][Payload] (Signature field is skipped)
        const signData = new Uint8Array(5 + 4 + payloadSize);
        signData[0] = 0x02;
        new DataView(signData.buffer).setUint32(1, totalSize - 5);
        new DataView(signData.buffer).setUint32(5, currentSeq);
        signData.set(buf.slice(payloadOffset), 9);
        
        const signature = await crypto.subtle.sign({ name: "HMAC" }, window.nhtml_crypto_key, signData);
        buf.set(new Uint8Array(signature), 9);
    }

    console.log(`🛰️ NHTML Sending Event: ${handler} (Seq: ${currentSeq}, Transport: ${window.nhtml_transport_mode})`);
    if (window.nhtml_transport_mode === "WASM") {
        wasmRunEvent(id_num, handler, formData);
    } else if (window.nhtml_transport_mode === "WS_READY") {
        sendBinary(buf);
    } else {
        console.warn("🛰️ NHTML Event dropped: Transport not ready (" + window.nhtml_transport_mode + ")");
    }
}

async function wasmRunEvent(id_num, handler, formData) {
    if (!window.nhtml_wasm_php) return;
    
    // Prepare PHP context
    const eventPayload = JSON.stringify({
        handler: handler,
        payload: JSON.stringify(formData),
        transport: window.nhtml_transport_mode
    });
    
    try {
        const php = await window.nhtml_wasm_php.binary;
        
        // Use a virtual file instead of stdin to avoid stream exhaustion issues
        php.FS.writeFile('/tmp/event.json', eventPayload);
        
        // Inject $input variable into the global scope before including app.php
        const phpCode = `<?php 
            $input = file_get_contents('/tmp/event.json');
            chdir('/'); 
            include 'app.php'; 
        ?>`;
        
        // Run and capture output
        let stdout = "";
        let stderr = "";
        const outListener = (event) => { 
            stdout += Array.isArray(event.detail) ? event.detail[0] : event.detail; 
        };
        const errListener = (event) => { 
            stderr += Array.isArray(event.detail) ? event.detail[0] : event.detail; 
        };
        
        window.nhtml_wasm_php.addEventListener('output', outListener);
        window.nhtml_wasm_php.addEventListener('error', errListener);
        
        await window.nhtml_wasm_php.run(phpCode);
        
        window.nhtml_wasm_php.removeEventListener('output', outListener);
        window.nhtml_wasm_php.removeEventListener('error', errListener);
        
        if (stderr) console.error("🛰️ NHTML PHP Error:", stderr);
        
        // Parse JSON output
        const jsonStart = stdout.indexOf('{');
        const jsonEnd = stdout.lastIndexOf('}');
        if (jsonStart !== -1 && jsonEnd !== -1) {
            const jsonStr = stdout.substring(jsonStart, jsonEnd + 1);
            console.log("🛰️ NHTML WASM RAW:", jsonStr);
            const res = JSON.parse(jsonStr);
            applyJsonPatch(res.patch || res);
        } else if (stdout.trim()) {
            console.log("🛰️ NHTML WASM STDOUT (Non-JSON):", stdout);
        }
    } catch (e) {
        console.error("🛰️ NHTML WASM Execution error:", e);
    }
}

function processEvent(e, listenFlag, fallbackAttr) {
    console.log(`🛰️ NHTML Event: ${e.type} on`, e.target, "Fallback:", fallbackAttr);
    
    // 1. Find the element that actually has the trigger attribute (n-click, n-input, etc.)
    const triggerAttr = Array.isArray(fallbackAttr) 
        ? (fallbackAttr.find(attr => e.target.closest(`[${attr}]`)))
        : fallbackAttr;
    
    const triggerEl = triggerAttr ? e.target.closest(`[${triggerAttr}]`) : null;
    
    // 2. Find the associated n-id (on the trigger element itself or a parent)
    const target = triggerEl || e.target.closest('[n-id]');
    
    if (!target) {
        console.log("  -> No NHTML target found.");
        return;
    }
    
    const nidEl = target.closest('[n-id]');
    const nid = nidEl ? nidEl.getAttribute('n-id') : null;
    console.log(`  -> Target Found: ${nid || 'no-id'}`, target);
    const id_num = window.nhtml_reverse_map[nid] || 0;
    const binding = window.nhtml_bindings && window.nhtml_bindings[id_num];
    
    if (binding) {
        // Only enforce listenMask if the event target is the EXACT element of the binding
        // or if the target doesn't have an explicit handler attribute.
        const hasExplicitAttr = Array.isArray(fallbackAttr) 
            ? fallbackAttr.some(attr => target.hasAttribute(attr))
            : target.hasAttribute(fallbackAttr);

        if (!hasExplicitAttr && !(binding.listenMask & listenFlag)) return;
        if (binding.behaviorFlags & 0x02) e.preventDefault(); // FLAG_N_PREVENT
    } else {
        // Fallback for dynamic elements (APPEND_HTML)
        const hasAttr = Array.isArray(fallbackAttr) 
            ? fallbackAttr.some(attr => target.hasAttribute(attr))
            : target.hasAttribute(fallbackAttr);
        if (!hasAttr) return;
    }
    
    // Handler Resolution: 
    // 1. Try target's own attribute (e.g. n-click)
    let handler = Array.isArray(fallbackAttr)
        ? (target.getAttribute(fallbackAttr[0]) || target.getAttribute(fallbackAttr[1]))
        : target.getAttribute(fallbackAttr);
        
    // 2. If no direct attribute, fallback to binding handler
    if (!handler && binding) {
        handler = binding.handler;
    }
    
    if (!handler) return;
    
    const formData = {};
    if (e.type === 'click') {
        formData.event_key = "";
        document.querySelectorAll('[n-id]').forEach(el => {
            if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
                formData[el.getAttribute('n-id')] = el.value;
            }
        });
    } else if (nid) {
        formData[nid] = target.value;
        if (e.type === 'keydown') formData.event_key = e.key;
    }
    
    const debounceMs = binding ? binding.debounce : 0;
    if (debounceMs > 0) {
        const timerKey = `${id_num}_${e.type}`;
        clearTimeout(window.nhtml_debounce_timers[timerKey]);
        window.nhtml_debounce_timers[timerKey] = setTimeout(() => {
            sendEvent(id_num, handler, formData);
        }, debounceMs);
    } else {
        sendEvent(id_num, handler, formData);
    }
}

document.addEventListener('click', (e) => processEvent(e, 0x01, 'n-click'));
document.addEventListener('input', (e) => processEvent(e, 0x02, ['n-input', 'n-change']));
document.addEventListener('keydown', (e) => processEvent(e, 0x08, 'n-keydown'));

function nhtml_startup() {
    // Scan existing IDs for hybrid mode (DevTools)
    document.querySelectorAll('[n-id]').forEach(el => {
        const nid = el.getAttribute('n-id');
        const id_num = parseInt(nid);
        if (!isNaN(id_num)) {
            window.nhtml_node_map[id_num] = nid;
            window.nhtml_reverse_map[nid] = id_num;
        }
    });

    const host = window.location.hostname || "127.0.0.1";
    const port = window.location.port || 8080;
    const path = window.location.pathname.substring(1);
    initNhtml(`ws://${host}:${port}/ws?sid=AUTO&path=${path}`);
}

if (document.readyState === 'loading') {
    window.addEventListener('DOMContentLoaded', nhtml_startup);
} else {
    nhtml_startup();
}
}
