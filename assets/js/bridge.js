if (window.nhtml_initialized) {
    console.warn("🛰️ NHTML Bridge already initialized.");
} else {
window.nhtml_initialized = true;
console.log("🛰️ NHTML v0.4.0 Bridge Loading...");

let socket = null;
let transportMode = "WS";
let httpFallbackUrl = "";
let reconnectAttempts = 0;
const MAX_BACKOFF = 30000;

window.nhtml_stats = { pkts: 0, size: 0 };
window.nhtml_node_map = {};
window.nhtml_reverse_map = {};
window.nhtml_el_cache = new Map(); // Cache des éléments pour performance
window.nhtml_session_id = localStorage.getItem('nhtml_session_id') || "";

async function initNhtml(wsUrl, httpUrl = "") {
    // Protocol fix: if we are on HTTPS, we MUST use WSS for the primary attempt
    if (window.location.protocol === "https:" && wsUrl.startsWith("ws:")) {
        wsUrl = wsUrl.replace("ws:", "wss:");
    }
    
    console.log("🛰️ NHTML Initializing with WS:", wsUrl);
    httpFallbackUrl = httpUrl;
    if (!window.nhtml_session_id || window.nhtml_session_id.length < 5) {
        window.nhtml_session_id = crypto.randomUUID().replace(/-/g, '');
        localStorage.setItem('nhtml_session_id', window.nhtml_session_id);
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

function nhtml_run_mousemove_actions(entry, e) {
    if (entry._done) return;
    let x = e.clientX, y = e.clientY;
    if (entry.la.flags & 0x04) { // SCOPE_SELF
        const rect = entry.el.getBoundingClientRect();
        x -= rect.left; y -= rect.top;
    }
    
    if (entry.la.actionType === 0x06) entry.el.style.setProperty(`--${entry.la.param}`, x);
    else if (entry.la.actionType === 0x07) entry.el.style.setProperty(`--${entry.la.param}`, y);
    else if (entry.la.actionType === 0x08) {
        // Distance
        const rect = entry.el.getBoundingClientRect();
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

        if (type === 0x09) { // PING
            const pong = new Uint8Array(5); pong[0] = 0x09;
            if(socket && socket.readyState === 1) socket.send(pong);
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
                            el.insertAdjacentHTML('beforebegin', new TextDecoder().decode(data.slice(2)));
                            break;
                        case 0x07: // INSERT_AFTER
                            opName = "INSERT_AFTER";
                            el.insertAdjacentHTML('afterend', new TextDecoder().decode(data.slice(2)));
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
                            el.innerHTML = new TextDecoder().decode(data.slice(2));
                            break;
                        case 0x0B: // APPEND_HTML
                            opName = "APPEND_HTML";
                            el.insertAdjacentHTML('beforeend', new TextDecoder().decode(data.slice(2)));
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
    if (transportMode === "WASM") return; // Skip if already in WASM mode
    
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
            transportMode = "WS"; 
            reconnectAttempts = 0; 
            const hudMode = document.getElementById('nhtml-hud-mode');
            if (hudMode) hudMode.innerText = "(WS)";
            sendHello(); 
        };
        
        socket.onmessage = (e) => processMessage(e.data);
        
        socket.onerror = (err) => {
            clearTimeout(connTimeout);
            console.warn("🛰️ NHTML WebSocket Error, checking fallback...");
            switchToWasm();
        };

        socket.onclose = () => { 
            clearTimeout(connTimeout);
            if (transportMode === "WS") {
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

async function switchToWasm() {
    if (transportMode === "WASM") return;
    transportMode = "WASM";
    console.log("🛰️ NHTML Switching to ZERO-SERVER (WASM) Mode...");
    const hudMode = document.getElementById('nhtml-hud-mode');
    if (hudMode) hudMode.innerText = "(WASM)";

    // Update Showcase UI if present
    const statusBadge = document.querySelector('.status-badge');
    if (statusBadge) {
        statusBadge.innerHTML = '<div style="width: 8px; height: 8px; background: #38bdf8; border-radius: 50%; box-shadow: 0 0 10px #38bdf8;"></div> WASM Mode (Local)';
    }
    
    try {
        // Load PHP WASM if not already there
        if (!window.PhpWeb) {
            // Note: fzstd is not needed for JSON output but bridge.js uses it for B-TREE
            const module = await import('./php-wasm/PhpWeb.mjs');
            window.PhpWeb = module.PhpWeb;
        }
        
        window.nhtml_wasm_php = new window.PhpWeb({ persist: true });
        
        let wasmStdout = "";
        window.nhtml_wasm_php.addEventListener('output', (event) => {
            wasmStdout += Array.isArray(event.detail) ? event.detail[0] : event.detail;
        });

        window.nhtml_wasm_php.addEventListener('ready', async () => {
            console.log("🛰️ NHTML WASM VM Ready.");
            
            // Sync filesystem from IndexedDB
            const phpInstance = await window.nhtml_wasm_php.binary;
            if (phpInstance.persist) {
                await new Promise(r => phpInstance.FS.syncfs(true, r));
                console.log("🛰️ NHTML WASM Persistent FS Synced.");
            }

            // Trigger initial state hydration
            wasmRunEvent(0, 'init', {});
        });
        
        // Initialize Virtual Filesystem
        const php = await window.nhtml_wasm_php.binary;
        
        // Load the local app.php and the SDK
        const basePath = window.location.pathname.replace(/\/[^\/]*$/, '/');
        const appPath = basePath + 'app.php';
        
        // Try to fetch app.php
        const resApp = await fetch(appPath);
        if (resApp.ok) {
            const code = await resApp.text();
            php.FS.writeFile('/app.php', code);
        }

        // Preload SDK files
        const sdkFiles = [
            '../../sdk/php/src/Nhtml.php',
            '../../sdk/php/src/Patch.php'
        ];
        
        for(const f of sdkFiles) {
            try {
                const res = await fetch(basePath + f);
                if (res.ok) {
                    const c = await res.text();
                    // Create directory structure in WASM FS
                    const parts = f.split('/');
                    let cur = "";
                    for(let i=0; i<parts.length -1; i++) {
                        cur += (cur === "" ? "" : "/") + parts[i];
                        try { php.FS.mkdir(cur); } catch(e){}
                    }
                    php.FS.writeFile("/" + f, c);
                }
            } catch (e) {
                console.warn("🛰️ NHTML Preload fail:", f);
            }
        }

    } catch (e) {
        console.error("🛰️ NHTML WASM initialization failed:", e);
    }
}

function applyJsonPatch(ops) {
    if (!Array.isArray(ops)) return;
    console.log(`%c[WASM PATCH] %cApplying ${ops.length} operations`, 'color:#ff007f;font-weight:bold', 'color:#fff');
    
    ops.forEach(op => {
        const nid = op.nid;
        const el = document.querySelector(`[n-id="${nid}"]`);
        if (!el) return;
        
        switch (op.op) {
            case 'set_text':
                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') el.value = op.val;
                else el.textContent = op.val;
                break;
            case 'add_class': el.classList.add(op.val); break;
            case 'del_class': el.classList.remove(op.val); break;
            case 'set_attr': el.setAttribute(op.key, op.val); break;
            case 'del_attr': el.removeAttribute(op.key); break;
            case 'set_style': el.style[op.prop] = op.val; break;
            case 'replace_inner': el.innerHTML = op.val; break;
            case 'append_html': el.insertAdjacentHTML('beforeend', op.val); break;
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
    const hello = new Uint8Array(8 + sidBytes.length);
    const dv = new DataView(hello.buffer);
    hello[0] = 0x01;
    dv.setUint32(1, 3 + sidBytes.length);
    dv.setUint16(5, 5000);
    hello[7] = sidBytes.length;
    hello.set(sidBytes, 8);
    socket.send(hello);
}

function sendBinary(data) {
    if (socket && socket.readyState === 1) socket.send(data);
}
window.sendBinary = sendBinary;

window.nhtml_debounce_timers = {};

function sendEvent(id_num, handler, formData) {
    const jsonBytes = new TextEncoder().encode(JSON.stringify(formData));
    const hBytes = new TextEncoder().encode(handler);
    
    // Header (5) + Payload (4 + 1 + hLen + 2 + pLen)
    const payloadLen = 4 + 1 + hBytes.length + 2 + jsonBytes.length;
    const buf = new Uint8Array(1 + 4 + payloadLen);
    const view = new DataView(buf.buffer);
    
    buf[0] = 0x02; // Type EVENT
    view.setUint32(1, payloadLen); // Length (Total Payload)
    
    view.setUint32(5, id_num); // NodeID
    buf[9] = hBytes.length; // HandlerLen
    buf.set(hBytes, 10); // Handler
    
    const pOff = 10 + hBytes.length;
    view.setUint16(pOff, jsonBytes.length); // PayloadLen
    buf.set(jsonBytes, pOff + 2); // Payload
    
    if (Object.keys(formData).length > 0) console.dir(formData);
    
    if (transportMode === "WASM") {
        wasmRunEvent(id_num, handler, formData);
    } else {
        sendBinary(buf);
    }
}

async function wasmRunEvent(id_num, handler, formData) {
    if (!window.nhtml_wasm_php) return;
    
    // Prepare PHP context
    const eventPayload = JSON.stringify({
        handler: handler,
        payload: JSON.stringify(formData)
    });
    
    // Set stdin
    window.nhtml_wasm_php.inputString(eventPayload);
    
    // We must run in a way that respects the working directory
    const phpCode = `<?php chdir(dirname('/app.php')); include '/app.php'; ?>`;
    
    try {
        // Run and capture output
        let stdout = "";
        const listener = (event) => { 
            stdout += Array.isArray(event.detail) ? event.detail[0] : event.detail; 
        };
        window.nhtml_wasm_php.addEventListener('output', listener);
        
        await window.nhtml_wasm_php.run(phpCode);
        
        window.nhtml_wasm_php.removeEventListener('output', listener);
        
        // Parse JSON output
        const jsonStart = stdout.indexOf('{');
        const jsonEnd = stdout.lastIndexOf('}');
        if (jsonStart !== -1 && jsonEnd !== -1) {
            const jsonStr = stdout.substring(jsonStart, jsonEnd + 1);
            const res = JSON.parse(jsonStr);
            applyJsonPatch(res.patch || res);

            // Sync back to IDBFS for persistence
            const phpInstance = await window.nhtml_wasm_php.binary;
            if (phpInstance.persist) {
                phpInstance.FS.syncfs(false, (err) => {
                    if (err) console.warn("🛰️ NHTML Sync error:", err);
                });
            }
        }
    } catch (e) {
        console.error("🛰️ NHTML WASM Execution error:", e);
    }
}

function processEvent(e, listenFlag, fallbackAttr) {
    const target = e.target.closest('[n-id]');
    if (!target) return;
    
    const nid = target.getAttribute('n-id');
    const id_num = window.nhtml_reverse_map[nid] || 0;
    const binding = window.nhtml_bindings && window.nhtml_bindings[id_num];
    
    if (binding) {
        if (!(binding.listenMask & listenFlag)) return;
        if (binding.behaviorFlags & 0x02) e.preventDefault(); // FLAG_N_PREVENT
    } else {
        // Fallback for dynamic elements (APPEND_HTML)
        const hasAttr = Array.isArray(fallbackAttr) 
            ? fallbackAttr.some(attr => target.hasAttribute(attr))
            : target.hasAttribute(fallbackAttr);
        if (!hasAttr) return;
    }
    
    let handler = binding ? binding.handler : "";
    if (!handler) {
        handler = Array.isArray(fallbackAttr)
            ? (target.getAttribute(fallbackAttr[0]) || target.getAttribute(fallbackAttr[1]))
            : target.getAttribute(fallbackAttr);
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
    } else {
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

window.addEventListener('DOMContentLoaded', () => {
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
});
}
