import init, { compile_wasm } from '../nhtml-core/pkg/nhtml_core.js';

class NhtmlEngine {
    constructor() {
        this.initialized = false;
        this.runtime_injected = false;
    }

    async init() {
        if (this.initialized) return;
        await init();
        this.initialized = true;
    }

    /**
     * Compile et monte un composant Nhtml dans un élément cible
     */
    async mount(selector, source) {
        await this.init();
        const result_json = compile_wasm(source);
        const result = JSON.parse(result_json);
        
        const target = document.querySelector(selector);
        if (target) {
            target.innerHTML = result.html;
            this.hydrate(result.manifest);
        }
    }

    /**
     * Système de réactivité léger (JS Bridge)
     */
    hydrate(manifest) {
        window._nhtmlAST = manifest;
        
        // Initialisation de l'état
        window.nhtml = window.nhtml || {};
        for (const [key, val] of Object.entries(manifest.state)) {
            if (!(key in window.nhtml)) {
                window.nhtml[key] = val;
            }
        }

        // Proxy pour réactivité automatique
        const self = this;
        const handler = {
            set(target, prop, value) {
                target[prop] = value;
                self.updateDOM();
                return true;
            }
        };
        window.nhtml = new Proxy(window.nhtml, handler);

        // Liaison des événements
        this.bindEvents(manifest.nodes);
        this.updateDOM();
    }

    bindEvents(nodes) {
        for (const [id, node] of Object.entries(nodes)) {
            if (node.events) {
                const el = document.getElementById(id);
                if (!el) continue;
                for (const [event, ops] of Object.entries(node.events)) {
                    el.addEventListener(event, (e) => {
                        for (const op of ops) {
                            if (op.op === 'increment') window.nhtml[op.target] += Number(op.value);
                            else if (op.op === 'set') {
                                let v = op.value;
                                if (v === "this.value") v = e.target.value;
                                window.nhtml[op.target] = v;
                            }
                        }
                    });
                }
            }
        }
    }

    updateDOM() {
        if (!window._nhtmlAST) return;
        for (const [id, node] of Object.entries(window._nhtmlAST.nodes)) {
            const el = document.getElementById(id);
            if (!el) continue;

            if (node.Text) {
                el.innerText = this.evalExpr(node.Text.expr);
            } else if (node.If) {
                const visible = !!this.evalExpr(node.If.condition);
                el.style.display = visible ? '' : 'none';
            }
        }
    }

    evalExpr(expr) {
        try {
            // Évaluation sécurisée via le scope window.nhtml
            const func = new Function('state', `with(state) { return ${expr}; }`);
            return func(window.nhtml);
        } catch (e) {
            return expr;
        }
    }
}

// Export global et auto-scan
const nhtml = new NhtmlEngine();
window.nhtmlEngine = nhtml;

// Auto-scan des balises Nhtml au chargement
window.addEventListener('DOMContentLoaded', async () => {
    const scripts = document.querySelectorAll('script[type="text/nhtml"]');
    for (const script of scripts) {
        const source = script.innerText || (script.src ? await (await fetch(script.src)).text() : "");
        const targetSelector = script.getAttribute('data-target') || 'body';
        if (source) {
            await nhtml.mount(targetSelector, source);
        }
    }
});
