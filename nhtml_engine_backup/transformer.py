import re
import json

from nhtml_engine.utils import dict_to_html_attrs

NHTML_RUNTIME_JS = r"""
// ── Nhtml Runtime ──────────────────────────────────────────────────────────
for (const p of (window._nhtmlPersistent || [])) {
    const saved = localStorage.getItem('nhtml_' + p);
    if (saved !== null) {
        try { window._nhtmlState[p] = JSON.parse(saved); } 
        catch(e) { window._nhtmlState[p] = saved; }
    }
}

window._nhtmlUpdateDOM = function() {
    document.querySelectorAll('[data-nhtml-text]').forEach(el => {
        try { el.textContent = window._nhtmlInterpolate(el.getAttribute('data-nhtml-text')); }
        catch(e) { console.warn('[Nhtml] text error:', e.message); }
    });
    document.querySelectorAll('[data-nhtml-html]').forEach(el => {
        try { el.innerHTML = window._nhtmlInterpolate(el.getAttribute('data-nhtml-html')); }
        catch(e) { console.warn('[Nhtml] html error:', e.message); }
    });
    document.querySelectorAll('[data-nhtml-attrs]').forEach(el => {
        const attrsJson = el.getAttribute('data-nhtml-attrs');
        let attrs;
        try { attrs = JSON.parse(attrsJson.replace(/&quot;/g, '"')); } catch(e) { return; }
        for (const [attr, template] of Object.entries(attrs)) {
            try {
                const value = window._nhtmlInterpolate(template);
                if (attr === 'color') el.style.color = value;
                else if (attr === 'background') el.style.background = value;
                else if (attr === 'disabled') el.disabled = (value === 'true' || value === true);
                else if (attr === 'visible') el.style.display = (value === 'false' || value === false) ? 'none' : '';
                else if (attr === 'value' && (el.tagName === 'INPUT' || el.tagName === 'SELECT' || el.tagName === 'TEXTAREA')) el.value = value;
                else el.setAttribute(attr, value);
            } catch(e) { console.warn('[Nhtml] attr error on', attr, ':', e.message); }
        }
    });
    document.querySelectorAll('[data-nhtml-if]').forEach(el => {
        try { el.style.display = window._nhtmlEval(el.getAttribute('data-nhtml-if')) ? '' : 'none'; }
        catch(e) { console.warn('[Nhtml] if error:', e.message); }
    });
    try {
        const titleTemplate = document.querySelector('title')?.getAttribute('data-nhtml-text');
        if (titleTemplate) document.title = window._nhtmlInterpolate(titleTemplate);
    } catch(e) {}

    for (const [varName, callbacks] of Object.entries(window._nhtmlWatchers || {})) {
        callbacks.forEach(cb => { try { cb(window._nhtmlState[varName]); } catch(e) {} });
    }
};

window._nhtmlInterpolate = function(template) {
    return template.replace(/\{(.*?)\}/g, (match, expr) => {
        const trimmed = expr.trim();
        if (/^['"]?\w+['"]?\s*:/.test(trimmed) && !trimmed.includes('?')) return match;
        return window._nhtmlEval(trimmed);
    });
};

window._nhtmlEval = function(expr) {
    try {
        const fn = new Function(...Object.keys(window._nhtmlState), `return ${expr}`);
        return fn(...Object.values(window._nhtmlState));
    } catch(e) {
        console.warn(`[Nhtml] Erreur d'évaluation pour l'expression '` + expr + `':`, e.message);
        return '';
    }
};

window.nhtml = new Proxy(window._nhtmlState, {
    set(target, prop, value) {
        target[prop] = value;
        if ((window._nhtmlPersistent || []).includes(prop)) {
            localStorage.setItem('nhtml_' + prop, JSON.stringify(value));
        }
        window._nhtmlUpdateDOM();
        document.dispatchEvent(new CustomEvent('nhtml:update', { detail: { prop, value } }));
        return true;
    },
    get(target, prop) {
        return target[prop];
    }
});

window._nhtmlWatch = function(varName, callback) {
    if (!window._nhtmlWatchers[varName]) window._nhtmlWatchers[varName] = [];
    window._nhtmlWatchers[varName].push(callback);
};

document.addEventListener('DOMContentLoaded', window._nhtmlUpdateDOM);
"""

NHTML_RUNTIME_V2 = r"""
// ── Nhtml V2 Headless Runtime (Micro-Runtime) ──────────────────────────────
(function() {
    const ast = window._nhtmlAST || { state_vars: {}, nodes: {} };
    window._nhtmlState = {};
    
    // 1. Initialisation du State
    for (const [key, config] of Object.entries(ast.state_vars || {})) {
        let val = config.initial_value;
        if (config.persist) {
            try {
                const saved = localStorage.getItem('nhtml_' + key);
                if (saved !== null) {
                    try { val = JSON.parse(saved); } catch(e) { val = saved; }
                }
            } catch(e) { console.warn("LocalStorage indisponible (mode file://)"); }
        }
        window._nhtmlState[key] = val;
    }

    // Evaluateur sécurisé
    window._nhtmlEval = function(expr, localContext = {}) {
        try {
            const scope = { ...window._nhtmlState, ...localContext };
            const fn = new Function(...Object.keys(scope), `return ${expr}`);
            return fn(...Object.values(scope));
        } catch(e) {
            console.warn("[Nhtml Eval Error]", expr, e.message);
            return "";
        }
    };

    // Exposer globalement (compatibilité)
    for (const key of Object.keys(window._nhtmlState)) {
        Object.defineProperty(window, key, {
            get() { return window._nhtmlState[key]; },
            set(v) { window._nhtmlState[key] = v; window._nhtmlHydrate(); }
        });
    }

    // Chargement de la persistance (localStorage)
    for (let prop in ast.state_vars) {
        if (ast.state_vars[prop].persist) {
            const saved = localStorage.getItem('nhtml_' + prop);
            if (saved !== null) {
                try { 
                    const parsed = JSON.parse(saved);
                    window._nhtmlState[prop] = parsed;
                    ast.state_vars[prop].initial_value = parsed;
                } catch(e){}
            }
        }
    }

    // Proxy Réactif
    window.nhtml = new Proxy(window._nhtmlState, {
        set(target, prop, value) {
            target[prop] = value;
            if (ast.state_vars[prop] && ast.state_vars[prop].persist) {
                try { localStorage.setItem('nhtml_' + prop, JSON.stringify(value)); } catch(e){}
            }
            window._nhtmlHydrate();
            return true;
        },
        get(target, prop) {
            return target[prop];
        }
    });

    // Moteur d'Event / OpCodes
    function runOpCodes(ops, localContext = {}) {
        const scope = { ...window._nhtmlState, ...localContext };
        for (const op of ops) {
            try {
                if (op.op === 'increment') window.nhtml[op.target] += Number(op.value);
                else if (op.op === 'decrement') window.nhtml[op.target] -= Number(op.value);
                else if (op.op === 'set') {
                    let v = op.value;
                    if (v === "this.value" && localContext.$event) v = localContext.$event.target.value;
                    else if (typeof v === "string" && isNaN(v)) v = window._nhtmlEval(v, localContext);
                    
                    // Support du binding profond (ex: "post.title")
                    if (op.target.includes('.')) {
                        const parts = op.target.split('.');
                        let current = window.nhtml;
                        for (let i = 0; i < parts.length - 1; i++) {
                            current = current[parts[i]];
                        }
                        current[parts[parts.length - 1]] = v;
                    } else {
                        window.nhtml[op.target] = v;
                    }
                }
                else if (op.op === 'call') {
                    const evalArgs = op.args.map(a => window._nhtmlEval(a, localContext));
                    if (typeof window[op.fn] === 'function') window[op.fn](...evalArgs);
                    else console.warn('Fonction inconnue:', op.fn);
                }
                else if (op.op === 'eval') {
                    window._nhtmlEval(op.expr, localContext);
                }
            } catch (e) { console.warn("[Nhtml OpCode Error]", op, e); }
        }
    }

    const ifGroupsMemory = {};

    // Hydratation & Diffing
    window._nhtmlHydrate = function() {
        for (const [id, node] of Object.entries(ast.nodes || {})) {
            const el = document.getElementById(id);
            if (!el) continue;

            if (node.type === "text") {
                el.textContent = window._nhtmlEval(node.expr);
            }
            else if (node.type === "html") {
                el.innerHTML = window._nhtmlEval(node.expr);
            }
            else if (node.type === "attrs" && node.bindings) {
                for (const [attr, val] of Object.entries(node.bindings)) {
                    let finalVal = val;
                    if (typeof val === 'string' && val.includes('{')) {
                        finalVal = val.replace(/\{(.*?)\}/g, (_, e) => window._nhtmlEval(e));
                    } else {
                        finalVal = window._nhtmlEval(val);
                    }

                    if (attr === 'disabled') el.disabled = !!finalVal;
                    else if (attr === 'value') el.value = finalVal;
                    else el.setAttribute(attr, finalVal);
                }
            }
            else if (node.type === "if") {
                // Logique de groupe simplifiée
                if (!ifGroupsMemory[node.group]) ifGroupsMemory[node.group] = { matched: false };
                
                if (node.role === 'if') {
                    const res = window._nhtmlEval(node.condition);
                    el.style.display = res ? '' : 'none';
                    ifGroupsMemory[node.group].matched = res;
                } else if (node.role === 'elseif') {
                    if (ifGroupsMemory[node.group].matched) { el.style.display = 'none'; }
                    else {
                        const res = window._nhtmlEval(node.condition);
                        el.style.display = res ? '' : 'none';
                        ifGroupsMemory[node.group].matched = res;
                    }
                } else if (node.role === 'else') {
                    el.style.display = ifGroupsMemory[node.group].matched ? 'none' : '';
                }
            }
            else if (node.type === "each") {
                let items = window._nhtmlEval(node.expr_in) || [];
                if (node.expr_filter) {
                    items = items.filter((item, i) => window._nhtmlEval(node.expr_filter, { [node.expr_as]: item, [node.expr_index]: i }));
                }
                
                el.innerHTML = "";
                items.forEach((item, i) => {
                    let html = node.template;
                    html = html.replace(/\[\[(.*?)\]\]/g, (_, expr) => {
                         let val = window._nhtmlEval(expr, { [node.expr_as]: item, [node.expr_index]: i });
                         return val;
                    });
                    el.insertAdjacentHTML('beforeend', html);
                });
            }
        }
    };

    // Binding des événements Initiaux
    document.addEventListener("DOMContentLoaded", () => {
        for (const [id, node] of Object.entries(ast.nodes || {})) {
            const el = document.getElementById(id);
            if (!el) continue;
            if (node.type === "attrs" && node.events) {
                for (const [evtName, ops] of Object.entries(node.events)) {
                    el.addEventListener(evtName, (e) => {
                        runOpCodes(ops, { $event: e });
                    });
                }
            }
        }
        window._nhtmlHydrate();
    });
})();
"""

class NhtmlTransformer:
    """
    Construit l'Arbre Syntaxique Abstrait (AST) de Nhtml.
    Au lieu de générer du JavaScript, chaque méthode transform_*
    compile l'intention déclarative dans self.ast_nodes.
    """

    def __init__(self, runtime_mode='inline'):
        self.runtime_mode = runtime_mode
        self.vars = {}           # state_vars
        self.persistent_vars = set() 
        self.components = {}     
        self.imports = {}        
        self.ast_nodes = {}      # AST Dictionnaire
        self.css_blocks = []     # blocs CSS à injecter
        self.element_counter = 0 # ids uniques

    def unique_id(self, prefix="nhtml"):
        self.element_counter += 1
        return f"{prefix}_{self.element_counter}"

    # ── VARIABLES RÉACTIVES ──────────────────────────────────────────────────

    def transform_var(self, attrs: dict) -> str:
        for name, value in attrs.items():
            if name == "nhtml_processed":
                continue
            self.vars[name] = value
        return ""

    def baseSetup(self) -> str:
        """
        Exporte le Manifest JSON `window._nhtmlAST` contenant l'Arbre d'Etat complet.
        Et charge le Micro-Runtime v2.
        """
        manifest = {
            "state_vars": {},
            "nodes": self.ast_nodes
        }
        for k, v in self.vars.items():
            manifest["state_vars"][k] = {
                "initial_value": v,
                "persist": (k in self.persistent_vars)
            }
        
        ast_json = json.dumps(manifest, ensure_ascii=False)
        
        state_setup = f"\nwindow._nhtmlAST = {ast_json};\n"
        
        return f"""
<script>
{state_setup}
{NHTML_RUNTIME_V2}
</script>
"""

    # ── LIAISON TEXTE {var} ──────────────────────────────────────────────────

    def transform_text_binding(self, tag: str, attrs: dict, content: str = "") -> str:
        """
        <h1 text="{title}"> → <h1 id="nhtml_x"></h1>
        """
        text_val = attrs.pop("text", None)
        html_val = attrs.pop("html", None)
        html_attrs = dict_to_html_attrs(attrs)
        
        uid = self.unique_id("n")
        
        if html_val is not None:
            self.ast_nodes[uid] = {"type": "html", "expr": str(html_val)}
            return f'<{tag} id="{uid}" {html_attrs}>{content}</{tag}>'
        if text_val is not None:
            self.ast_nodes[uid] = {"type": "text", "expr": str(text_val)}
            return f'<{tag} id="{uid}" {html_attrs}>{content}</{tag}>'
        return f'<{tag} {html_attrs}>{content}</{tag}>'

    # ── ÉVÉNEMENTS on: ───────────────────────────────────────────────────────

    def transform_events(self, attrs: dict) -> dict:
        """
        on:click="expr" → onclick="expr"
        Convertit les attributs d'événements Nhtml en attributs HTML standards.
        """
        new_attrs = {}
        for k, v in attrs.items():
            if k.startswith("on:"):
                event = k[3:]
                new_attrs[f"on{event}"] = v
            else:
                new_attrs[k] = v
        return new_attrs

    # ── CONDITIONNELS if= (attribut inline) ─────────────────────────────────

    def transform_if_block(self, condition: str, content: str, elseif_blocks: list = None, else_content: str = "") -> str:
        """
        Reçoit la structure complète if/elseif/else du parser et génère le HTML pur + AST.
        (condition, if_content, elseif_blocks=[(cond, content), ...], else_content)
        """
        group_id = self.unique_id("nhif")
        cond = condition.strip().replace('{', '').replace('}', '')

        uid_if = self.unique_id("n")
        self.ast_nodes[uid_if] = {
            "type": "if",
            "group": group_id,
            "role": "if",
            "condition": cond
        }
        html = f'<div id="{uid_if}" style="display:none">{content}</div>'

        if elseif_blocks:
            for ei_cond, ei_content in elseif_blocks:
                ei_uid = self.unique_id("n")
                ei_c = ei_cond.strip().replace('{', '').replace('}', '')
                self.ast_nodes[ei_uid] = {
                    "type": "if",
                    "group": group_id,
                    "role": "elseif",
                    "condition": ei_c
                }
                html += f'\n<div id="{ei_uid}" style="display:none">{ei_content}</div>'

        if else_content:
            el_uid = self.unique_id("n")
            self.ast_nodes[el_uid] = {
                "type": "if",
                "group": group_id,
                "role": "else"
            }
            html += f'\n<div id="{el_uid}" style="display:none">{else_content}</div>'

        return html

    def transform_each(self, in_var, as_name: str, index_name: str, filter_expr: str, template: str, container_tag: str = "div") -> str:
        """
        <each in="{produits}" as="produit" index="i" filter="..." tag="tbody">
        → Genère un conteneur et remplit l'AST pour le moteur
        """
        uid = self.unique_id("n")
        in_var = str(in_var)
        if in_var.startswith('{') and in_var.endswith('}'):
            in_var = in_var[1:-1]

        # Convertir les {expr} en [[expr]] dans le template pour l'AST JSON
        # On ne veut remplacer QUE les textes d'interpolation
        def replace_brackets(m):
            return f"[[{m.group(1)}]]"
        safe_template = re.sub(r'\{([^{}\n\r]+?)\}', replace_brackets, template)

        # Enregistrer le noeud AST
        self.ast_nodes[uid] = {
            "type": "each",
            "expr_in": in_var,
            "expr_as": as_name,
            "expr_index": index_name or "i",
            "expr_filter": filter_expr,
            "template": safe_template
        }
        
        # Le html retourné est simplement le conteneur vide
        return f'<{container_tag} id="{uid}"></{container_tag}>'

    def transform_for_each(self, tag: str, attrs: dict) -> str:
        """
        Non supporté dans la v2 pour le moment, à remplacer par <each>.
        """
        return ""

    # ── FORMULAIRES bind: et validate: ───────────────────────────────────────

    def transform_input(self, tag: str, attrs: dict) -> str:
        """
        bind:value="{email}" → oninput="email = this.value"
        Conserve la balise d'origine (input, select, textarea).
        """
        bind_value = attrs.pop("bind:value", None)
        validations = {}

        keys_to_remove = []
        for k in list(attrs.keys()):
            if k.startswith("validate:"):
                rule = k[len("validate:"):]
                validations[rule] = attrs[k]
                keys_to_remove.append(k)
        for k in keys_to_remove:
            del attrs[k]

        # Liaison bidirectionnelle
        if bind_value:
            var_name = bind_value.strip("{} ")
            # On ajoute oninput en préservant l'existant
            existing_oninput = attrs.get("oninput", "")
            attrs["oninput"] = (existing_oninput + f"; {var_name} = this.value").strip("; ")
            
            # Utiliser data-nhtml-attrs pour que _nhtmlUpdateDOM résolve la valeur au chargement
            attrs["data-nhtml-attrs"] = json.dumps({"value": f"{{{var_name}}}"}).replace('"', '&quot;')

        # Validation HTML5 native
        if "required" in validations:
            attrs["required"] = True
        if "min" in validations:
            attrs["minlength"] = validations["min"]
        if "max" in validations:
            attrs["maxlength"] = validations["max"]
        if "pattern" in validations:
            attrs["pattern"] = validations["pattern"]
        if "message" in validations:
            attrs["title"] = validations["message"]

        html_attrs = dict_to_html_attrs(attrs)
        
        # On ne renvoie QUE la balise ouvrante modifiée.
        # Le reste du contenu et la balise fermante d'origine sont déjà dans le source.
        if tag in ("input", "br", "hr", "img"):
            return f"<{tag} {html_attrs}/>"
        return f"<{tag} {html_attrs}>"

    # ── COMPOSANTS ───────────────────────────────────────────────────────────

    def register_component(self, name: str, content: str, props: list):
        """Enregistre un composant défini avec <component name='...'>"""
        self.components[name] = {"content": content, "props": props}

    def instantiate_component(self, name: str, attrs: dict) -> str:
        """Remplace <mon-composant prop1='...'> par son contenu."""
        if name not in self.components:
            return f"<!-- Composant '{name}' non trouvé -->"

        comp = self.components[name]
        content = comp["content"]

        # Injecter les props dans le template
        for prop_name, prop_value in attrs.items():
            content = content.replace(f"{{{prop_name}}}", str(prop_value))

        return content
