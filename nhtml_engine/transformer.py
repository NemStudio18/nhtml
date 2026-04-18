import re
import json

from nhtml_engine.utils import dict_to_html_attrs

class NhtmlTransformer:
    """
    Contient toutes les règles de transformation Nhtml.
    Chaque méthode transform_* gère un concept de la spec.
    """

    def __init__(self):
        self.vars = {}           # variables globales déclarées avec <var>
        self.persistent_vars = set() # variables persistées dans localStorage
        self.components = {}     # composants déclarés avec <component>
        self.imports = {}        # composants importés avec <import>
        self.js_blocks = []      # blocs JS à injecter
        self.css_blocks = []     # blocs CSS à injecter
        self.element_counter = 0 # pour générer des ids uniques

    def unique_id(self, prefix="nhtml"):
        self.element_counter += 1
        return f"{prefix}_{self.element_counter}"

    # ── VARIABLES RÉACTIVES ──────────────────────────────────────────────────

    def transform_var(self, attrs: dict) -> str:
        """
        <var compteur=0>
        → déclare une variable réactive globale
        → génère le JS de réactivité correspondant
        """
        for name, value in attrs.items():
            if name == "nhtml_processed":
                continue
            self.vars[name] = value

        # On génère le système de réactivité pour toutes les vars
        js = self._generate_reactivity_system()
        self.js_blocks = [js]  # on remplace (recalcul complet)
        return ""  # la balise <var> disparaît du HTML

    def _generate_reactivity_system(self) -> str:
        """
        Génère un système de réactivité simple basé sur des Proxies JS.
        Chaque modification d'une variable met à jour le DOM automatiquement.
        """
        if not self.vars:
            return ""

        vars_init = json.dumps(self.vars, ensure_ascii=False)
        persistent_list = json.dumps(list(self.persistent_vars))

        return f"""
// ── Système de réactivité Nhtml ──────────────────────────────────────────
const _nhtmlState = {vars_init};
const _nhtmlPersistent = {persistent_list};
const _nhtmlWatchers = {{}};
const _nhtmlBindings = [];

// Restaurer l'état persistant au démarrage
for (const p of _nhtmlPersistent) {{
    const saved = localStorage.getItem('nhtml_' + p);
    if (saved !== null) {{
        try {{
            _nhtmlState[p] = JSON.parse(saved);
        }} catch(e) {{
            _nhtmlState[p] = saved;
        }}
    }}
}}

function _nhtmlUpdateDOM() {{
    // Mettre à jour les éléments avec text="{{var}}"
    document.querySelectorAll('[data-nhtml-text]').forEach(el => {{
        const template = el.getAttribute('data-nhtml-text');
        el.textContent = _nhtmlInterpolate(template);
    }});
    // Mettre à jour les éléments avec html="{{var}}"
    document.querySelectorAll('[data-nhtml-html]').forEach(el => {{
        const template = el.getAttribute('data-nhtml-html');
        el.innerHTML = _nhtmlInterpolate(template);
    }});
    // Mettre à jour les attributs calculés
    document.querySelectorAll('[data-nhtml-attrs]').forEach(el => {{
        const attrsJson = el.getAttribute('data-nhtml-attrs');
        let attrs;
        try {{ attrs = JSON.parse(attrsJson.replace(/&quot;/g, '"')); }} catch(e) {{ return; }}
        for (const [attr, template] of Object.entries(attrs)) {{
            const value = _nhtmlInterpolate(template);
            if (attr === 'color') el.style.color = value;
            else if (attr === 'background') el.style.background = value;
            else if (attr === 'disabled') el.disabled = (value === 'true' || value === true);
            else if (attr === 'visible') el.style.display = (value === 'false' || value === false) ? 'none' : '';
            else if (attr === 'value' && (el.tagName === 'INPUT' || el.tagName === 'SELECT' || el.tagName === 'TEXTAREA')) el.value = value;
            else el.setAttribute(attr, value);
        }}
    }});
    // Mettre à jour les conditions if
    document.querySelectorAll('[data-nhtml-if]').forEach(el => {{
        const expr = el.getAttribute('data-nhtml-if');
        const result = _nhtmlEval(expr);
        el.style.display = result ? '' : 'none';
    }});
    // Mettre à jour le titre de la page si {{site_name}} ou autre
    const titleTemplate = document.querySelector('title')?.getAttribute('data-nhtml-text');
    if (titleTemplate) document.title = _nhtmlInterpolate(titleTemplate);

    // Déclencher les watchers
    for (const [varName, callbacks] of Object.entries(_nhtmlWatchers)) {{
        callbacks.forEach(cb => cb(_nhtmlState[varName]));
    }}
}}

function _nhtmlInterpolate(template) {{
    return template.replace(/\\{{(.*?)\\}}/g, (match, expr) => {{
        const trimmed = expr.trim();
        // Si ça ressemble à un objet JS literal {{a:1, b:2}}, on ne l'interpole pas
        // On vérifie stricto sensu la syntaxe d'un objet JS pour ne pas bloquer les "ternaires ?"
        if (/^['"]?\\w+['"]?\\s*:/.test(trimmed) && !trimmed.includes('?')) return match;
        return _nhtmlEval(trimmed);
    }});
}}

function _nhtmlEval(expr) {{
    try {{
        const fn = new Function(...Object.keys(_nhtmlState), `return ${{expr}}`);
        return fn(...Object.values(_nhtmlState));
    }} catch(e) {{
        console.warn(`[Nhtml] Erreur d'évaluation pour l'expression '` + expr + `':`, e.message);
        return '';
    }}
}}

// Proxy pour intercepter les modifications de variables
const nhtml = new Proxy(_nhtmlState, {{
    set(target, prop, value) {{
        target[prop] = value;
        // Sauvegarde si persistant
        if (_nhtmlPersistent.includes(prop)) {{
            localStorage.setItem('nhtml_' + prop, JSON.stringify(value));
        }}
        _nhtmlUpdateDOM();
        // Déclencher l'événement pour les groupes if/each
        document.dispatchEvent(new CustomEvent('nhtml:update', {{ detail: {{ prop, value }} }}));
        return true;
    }},
    get(target, prop) {{
        return target[prop];
    }}
}});

// Exposer les variables globalement
{self._expose_vars()}

function _nhtmlWatch(varName, callback) {{
    if (!_nhtmlWatchers[varName]) _nhtmlWatchers[varName] = [];
    _nhtmlWatchers[varName].push(callback);
}}

// Initialiser l'affichage au chargement
document.addEventListener('DOMContentLoaded', _nhtmlUpdateDOM);
// ─────────────────────────────────────────────────────────────────────────
"""

    def _expose_vars(self) -> str:
        """Expose chaque variable globalement via le proxy."""
        lines = []
        for name in self.vars:
            lines.append(
                f"Object.defineProperty(window, '{name}', {{"
                f"get() {{ return nhtml['{name}']; }},"
                f"set(v) {{ nhtml['{name}'] = v; }}"
                f"}});"
            )
        return "\n".join(lines)

    # ── LIAISON TEXTE {var} ──────────────────────────────────────────────────

    def transform_text_binding(self, tag: str, attrs: dict, content: str = "") -> str:
        """
        <h1 text="{title}"> → <h1><span data-nhtml-text="{title}">{title}</span></h1>
        """
        text_val = attrs.pop("text", None)
        html_val = attrs.pop("html", None)

        html_attrs = dict_to_html_attrs(attrs)
        
        if html_val is not None:
            val = str(html_val)
            return f'<{tag} {html_attrs} data-nhtml-html="{val}">{content}</{tag}>'
        if text_val is not None:
            val = str(text_val)
            return f'<{tag} {html_attrs}><span data-nhtml-text="{val}">{val}</span></{tag}>'
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

    def transform_if_attr(self, attrs: dict) -> dict:
        """
        <div if="is_admin"> → <div data-nhtml-if="is_admin" style="display:none">
        Attribut if= simple sur les balises HTML standard.
        """
        if "if" in attrs:
            condition = str(attrs.pop("if"))
            condition = condition.replace('{', '').replace('}', '')
            attrs["data-nhtml-if"] = condition
        return attrs

    # ── BLOCS if/elseif/else ─────────────────────────────────────────────────

    def transform_if_block(self, condition: str, content: str, elseif_blocks: list = None, else_content: str = "") -> str:
        """
        Reçoit la structure complète if/elseif/else du parser et génère le HTML + JS.
        (condition, if_content, elseif_blocks=[(cond, content), ...], else_content)
        """
        uid = self.unique_id("nhif")
        cond = condition.strip().replace('{', '').replace('}', '')

        html = f'<div data-nhtml-if="{cond}" data-nhtml-group="{uid}" style="display:none">{content}</div>'

        if elseif_blocks:
            for ei_cond, ei_content in elseif_blocks:
                ei_c = ei_cond.strip().replace('{', '').replace('}', '')
                html += f'\n<div data-nhtml-elseif="{ei_c}" data-nhtml-group="{uid}" style="display:none">{ei_content}</div>'

        if else_content:
            html += f'\n<div data-nhtml-else data-nhtml-group="{uid}" style="display:none">{else_content}</div>'

        # Générer le JS pour ce groupe
        self.transform_if_group(uid)
        return html

    def transform_if_group(self, uid: str) -> str:
        """Génère le JS pour gérer un groupe if/elseif/else."""
        js = (f"""
document.addEventListener('DOMContentLoaded', function() {{
    function _nhtmlUpdateGroup_{uid}() {{
        const group = document.querySelectorAll('[data-nhtml-group="{uid}"]');
        let matched = false;
        group.forEach(el => {{
            if (el.hasAttribute('data-nhtml-if')) {{
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            }} else if (el.hasAttribute('data-nhtml-elseif')) {{
                if (matched) {{ el.style.display = 'none'; }}
                else {{
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }}
            }} else if (el.hasAttribute('data-nhtml-else')) {{
                el.style.display = matched ? 'none' : '';
            }}
        }});
    }}
    _nhtmlUpdateGroup_{uid}();
    // Re-évaluer à chaque mise à jour du state
    const _orig_{uid} = _nhtmlUpdateDOM;
    const _prev_{uid} = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_{uid} = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_{uid});
}});
""")
        self.js_blocks.append(js)
        return ""

    # ── LISTES <each> ────────────────────────────────────────────────────────

    def transform_each(self, in_var, as_name: str, index_name: str, filter_expr: str, template: str, container_tag: str = "div") -> str:
        """
        <each in="{produits}" as="produit" index="i" filter="..." tag="tbody">
        → conteneur JS qui génère les éléments dynamiquement
        """
        uid = self.unique_id("nheach")
        in_var = str(in_var)
        if in_var.startswith('{') and in_var.endswith('}'):
            in_var = in_var[1:-1]

        filter_js = ""
        if filter_expr:
            f_expr = filter_expr.strip() if isinstance(filter_expr, str) else str(filter_expr)
            safe_f_expr = f_expr.replace("'", "\\'")
            filter_js = f"items = items.filter(({as_name}, {index_name or 'i'}) => _nhtmlEval('{safe_f_expr}'));"

        index_decl = f"const {index_name} = _i;" if index_name else ""

        # ── Scoping : on remplace as_name → _nhtml_loop_item_{uid} ────────
        def safe_scope_replace(val):
            pattern = (
                rf'(?<![\w.\-]){re.escape(as_name)}(?!\s*=>)(?=[\s.()\[\]!?&|><:=,`\'"])'
                rf'|(?<![\w.\-]){re.escape(as_name)}$'
            )
            return re.sub(pattern, f"_nhtml_loop_item_{uid}", val)

        # ── Conversion du template en template literal JS ──────────────────
        # Regex simple mais correcte : on cherche {expr} sans imbrication
        # Les attributs déjà encodés (&quot;) ne sont pas des {}, donc pas de conflit.
        parts = []
        last = 0
        for m in re.finditer(r'\{([^{}\n\r]+?)\}', template):
            before = template[last:m.start()]
            # Échapper les backticks et les ${ littéraux dans le texte HTML brut
            parts.append(before.replace('`', '\\`').replace('${', '\\${'))
            expr = m.group(1)
            parts.append('${' + safe_scope_replace(expr) + '}')
            last = m.end()
        # Reste du template après la dernière expression
        parts.append(template[last:].replace('`', '\\`').replace('${', '\\${'))
        scoped_js_template = "".join(parts)

        # Préparation des expressions pour injection sécurisée dans le JS
        safe_in_var = str(in_var).replace("'", "\\'")

        js = f"""
document.addEventListener('DOMContentLoaded', function() {{
    function _nhtmlRender_{uid}() {{
        const container = document.getElementById('{uid}');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {{
            items = _nhtmlEval('{safe_in_var}');
            if (!Array.isArray(items)) items = [];
        }} catch(e) {{
            console.warn("[Nhtml] Erreur boucle each:", e);
        }}
        
        {filter_js}

        container.innerHTML = '';
        items.forEach((_item, _i) => {{
            {index_decl}
            const _nhtml_loop_item_{uid} = _item;
            const _tpl = `{scoped_js_template}`;
            container.insertAdjacentHTML('beforeend', _tpl);
        }});
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="{uid}"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {{
            _nhtmlUpdateDOM();
        }}
    }}
    _nhtmlRender_{uid}();
    document.addEventListener('nhtml:update', _nhtmlRender_{uid});
}});
"""
        self.js_blocks.append(js)
        return f'<{container_tag} id="{uid}" data-nhtml-each="{in_var}"></{container_tag}>'

    def _template_to_js(self, template: str) -> str:
        """Convertit {var} en ${var} pour les template literals JS."""
        # Remplace {expr} en ${expr}, gère aussi !{var} -> ${!var}
        def replacer(m):
            expr = m.group(1)
            return '${' + expr + '}'
        return re.sub(r'\{([^{}\n\r]+?)\}', replacer, template)

    def transform_for_each(self, tag: str, attrs: dict) -> str:
        """
        <p for:each="{items}" as="item" text="{item}"/>
        → version simple de each pour une seule balise
        """
        in_var = attrs.pop("for:each", "").strip("{} ")
        as_name = attrs.pop("as", "item")
        index_name = attrs.pop("index", None)
        text_tpl = attrs.pop("text", "")

        uid = self.unique_id("nhforeach")
        index_decl = f"const {index_name} = _i;" if index_name else ""
        text_js = self._template_to_js(text_tpl)

        html_attrs = dict_to_html_attrs(attrs)
        js = f"""
document.addEventListener('DOMContentLoaded', function() {{
    function _nhtmlRender_{uid}() {{
        const container = document.getElementById('{uid}');
        if (!container) return;
        const items = nhtml['{in_var}'] || [];
        container.innerHTML = '';
        items.forEach(({as_name}, _i) => {{
            {index_decl}
            const el = document.createElement('{tag}');
            el.textContent = `{text_js}`;
            container.appendChild(el);
        }});
    }}
    _nhtmlRender_{uid}();
    document.addEventListener('nhtml:update', _nhtmlRender_{uid});
}});
"""
        self.js_blocks.append(js)
        return f'<div id="{uid}" data-nhtml-foreach="{in_var}"></div>'

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
