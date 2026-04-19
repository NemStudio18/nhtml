import re
import json
from nhtml_engine.utils import dict_to_html_attrs, parse_inline_attrs
from nhtml_engine.transformer import NhtmlTransformer


class NhtmlParser:
    """
    Parser v1.2 — Correctif définitif.
    Stratégie : Split simple sur `<tag>` (sans capturer les sous-groupes)
    pour isoler les nœuds texte, puis traitement de l'interpolation.
    """

    NHTML_TAGS = {"nhtml", "var", "component", "import", "each", "empty",
                  "if", "elseif", "else", "props", "prop", "slot"}

    def __init__(self, runtime_mode='inline'):
        self.transformer = NhtmlTransformer(runtime_mode=runtime_mode)

    def parse(self, source: str) -> str:
        source = re.sub(r'<!nhtml[^>]*>', '', source).strip()
        source = self._process_vars(source)
        source = self._process_components(source)
        source = self._instantiate_components(source)
        source = self._process_nhtml_attrs(source)
        source = self._process_title_tag(source)
        source = self._process_text_nodes(source)
        source = self._process_if_blocks(source)
        source = self._process_empty_tags(source)
        source = self._process_each_blocks(source)
        return self._assemble(source)

    def _process_empty_tags(self, source: str) -> str:
        """ Support de <empty for="#list">... </empty> """
        pattern = re.compile(r'<empty\s+for=["\']#?([^"\']+)["\'][^>]*>(.*?)</empty>', re.DOTALL | re.IGNORECASE)
        def replace_empty(m):
            var_name, content = m.group(1), m.group(2)
            # Transpilation en <if condition="var.length == 0">
            return f'<if condition="{var_name}.length === 0">{content}</if>'
        return pattern.sub(replace_empty, source)

    # ── 1. Variables ──────────────────────────────────────────────────────────

    def _process_vars(self, source: str) -> str:
        from nhtml_engine.utils import parse_inline_attrs
        # Regex V2 Robuste pour <var> : ne s'arrête pas au premier > s'il est dans un attribut
        var_pattern = re.compile(
            r'<var'
            r'((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)'
            r'\s*/?>',
            re.IGNORECASE | re.DOTALL
        )

        def replace_var(m):
            raw = m.group(1).strip()
            attrs = parse_inline_attrs(raw)
            
            is_persistent = "persist" in attrs
            if is_persistent: del attrs["persist"]

            for name, raw_value in attrs.items():
                if name == "persist": continue
                
                value = raw_value
                if isinstance(raw_value, str):
                    s = raw_value.strip()
                    if (s.startswith('{') and s.endswith('}')) or (s.startswith('[') and s.endswith(']')):
                        try:
                            value = json.loads(s)
                        except Exception:
                            value = s
                    elif s.lower() == "true": value = True
                    elif s.lower() == "false": value = False
                    elif s.isdigit(): value = int(s)
                    elif re.match(r'^-?\d+\.\d+$', s): value = float(s)
                
                self.transformer.transform_var({name: value})
                if is_persistent:
                    self.transformer.persistent_vars.add(name)
            return ""

        return var_pattern.sub(replace_var, source)

    # ── 2. Attributs dynamiques ───────────────────────────────────────────────

    def _process_nhtml_attrs(self, source: str) -> str:
        from nhtml_engine.utils import js_to_opcodes

        # On protège d'abord les blocs each pour ne pas polluer leurs attributs internes
        protected_each = []
        def protect_each(m):
            protected_each.append(m.group(0))
            return f"<!--NHEACHPROT{len(protected_each)-1}-->"
        
        source = re.sub(r'<each.*?>.*?</each>', protect_each, source, flags=re.DOTALL | re.IGNORECASE)

        # Regex pour trouver les balises avec des attributs Nhtml (on:, bind:, etc.)
        tag_re = re.compile(
            r'<(?!each|if|else|elseif)([\w.-]+)' # On ignore les balises de contrôle
            r'((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)'
            r'\s*(/?)>',
            re.DOTALL
        )

        def replace_tag(m):
            tag_name, attrs_raw, self_close = m.groups()
            if tag_name.lower() in self.NHTML_TAGS:
                return m.group(0)
            attrs = parse_inline_attrs(attrs_raw)
            dynamic_attrs, static_attrs, events = {}, {}, {}
            
            # Gestion explicite de bind:value
            bind_value = attrs.pop("bind:value", None)
            if bind_value:
                var_name = bind_value.strip("{} ")
                # OpCodes au format JSON plutôt que JS libre
                events["input"] = [{"op": "set", "target": var_name, "value": "this.value"}]
                dynamic_attrs["value"] = f"{{{var_name}}}"

            for k, v in attrs.items():
                ks, vs = str(k), str(v)
                if ks.startswith("on:"):
                    event_name = ks[3:] # click, submit...
                    events[event_name] = js_to_opcodes(vs)
                elif ks.startswith('{') and ks.endswith('}'):
                    dynamic_attrs[ks[1:-1]] = vs
                elif '{' in vs and '}' in vs and not ks.startswith('data-nhtml-'):
                    dynamic_attrs[ks] = vs
                else:
                    static_attrs[ks] = v

            static_str = dict_to_html_attrs(static_attrs)
            if not dynamic_attrs and not events:
                # Retirer juste l'appel à m.group(0) sinon static_str n'est pas appliqué s'il y a eu des binds sans evt
                # Mais si ni evt ni class, on retourne tel quel (ou on reformate, plus propre de formater)
                pass # on continue pour rajouter l'id si on en avait besoin, mais ici non.
                if m.group(2).strip() == static_str.strip():
                     return m.group(0)
                else:
                     return f'<{tag_name} {static_str}{self_close}>'

            uid = self.transformer.unique_id("n")
            ast_block = {"type": "attrs"}
            if dynamic_attrs:
                ast_block["bindings"] = dynamic_attrs
            if events:
                ast_block["events"] = events
            
            self.transformer.ast_nodes[uid] = ast_block

            return f'<{tag_name} id="{uid}" {static_str}{self_close}>'

        source = tag_re.sub(replace_tag, source)

        # Restauration des boucles protégées
        for i, block in enumerate(protected_each):
            source = source.replace(f"<!--NHEACHPROT{i}-->", block)
        
        return source

    # ── 3. Titre ──────────────────────────────────────────────────────────────

    def _process_title_tag(self, source: str) -> str:
        def replace_title(m):
            content = m.group(1).strip()
            if '{' in content:
                safe = content.replace('"', '&quot;')
                return f'<title data-nhtml-text="{safe}">{content}</title>'
            return m.group(0)
        return re.sub(r'<title>(.*?)</title>', replace_title, source,
                      flags=re.IGNORECASE | re.DOTALL)

    # ── 4. Interpolation texte — Split simple ─────────────────────────────────

    def _process_text_nodes(self, source: str) -> str:
        """
        Version définitive v1.2 :
        - Protège <script>, <style>, <title> entiers.
        - Utilise re.split avec UN SEUL groupe de capture (la balise),
          ce qui produit une liste alternée [texte, tag, texte, tag, ...].
        - N'applique l'interpolation qu'aux segments texte.
        """
        # 1. Protéger et COLLECTER les blocs verbatim (+ boucles pour éviter IDs globaux)
        protected = []
        def protect(m):
            tag = m.group(1).lower()
            content = m.group(2)
            if tag == 'style':
                self.transformer.css_blocks.append(content)
                return ""
            protected.append(m.group(0))
            return f'<!--NHPROT{len(protected)-1}-->'
        
        # On protège style, script, title ET les templates de boucles each
        source = re.sub(
            r'<(script|style|title|each)[^>]*>(.*?)</\1>',
            protect, source, flags=re.DOTALL | re.IGNORECASE
        )

        # Regex robuste pour split, utilisant UN SEUL groupe de capture englobant.
        # Ne s'arrête pas aux '>' situés dans les attributs.
        split_pattern = re.compile(
            r'(<[\w.-]+'
            r'(?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*'
            r'\s*/?>)',
            re.DOTALL
        )
        parts = split_pattern.split(source)

        # 3. Appliquer l'interpolation aux segments texte seulement
        expr_re = re.compile(r'\{([^{}]+)\}')

        def wrap_expr(m):
            expr = m.group(1).strip()
            # Ignorer les pseudo-sélecteurs CSS et les clés JSON sans ternaire
            if expr.startswith('/') or (': ' in expr and '?' not in expr):
                return m.group(0)
            uid = self.transformer.unique_id("n")
            self.transformer.ast_nodes[uid] = {"type": "text", "expr": expr}
            return f'<span id="{uid}"></span>'

        result = []
        for part in parts:
            if part and not part.startswith('<') and '{' in part:
                result.append(expr_re.sub(wrap_expr, part))
            else:
                result.append(part)

        source = ''.join(result)

        # 4. Restaurer les blocs protégés
        for i, block in enumerate(protected):
            source = source.replace(f'<!--NHPROT{i}-->', block)

        return source

    # ── 5. Composants ────────────────────────────────────────────────────────

    def _process_components(self, source: str) -> str:
        pattern = re.compile(
            r'<component\s+name=["\']([^"\']+)["\'][^>]*>(.*?)</component>',
            re.DOTALL | re.IGNORECASE
        )
        def replace_component(m):
            name, content = m.group(1), m.group(2).strip()
            props = self._extract_props(content)
            content = re.sub(r'<props>.*?</props>', '', content, flags=re.DOTALL).strip()
            self.transformer.register_component(name, content, props)
            return ''
        return pattern.sub(replace_component, source)

    def _extract_props(self, content: str) -> list:
        props = []
        pm = re.search(r'<props>(.*?)</props>', content, re.DOTALL)
        if pm:
            for p in re.finditer(r'<prop\s+([^>]+?)/?>', pm.group(1), re.IGNORECASE):
                props.append(parse_inline_attrs(p.group(1)))
        return props

    def _instantiate_components(self, source: str) -> str:
        for comp_name in self.transformer.components.keys():
            # Forme avec slot: <ComponentName attr="val">...</ComponentName>
            pattern = re.compile(
                fr'<{comp_name}\s*((?:[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{{[^}}]*}}|[\w./:-]+))?\s*)*)>(.*?)</{comp_name}>',
                re.DOTALL | re.IGNORECASE
            )
            def replace_instance(m):
                attrs = parse_inline_attrs(m.group(1))
                slot_content = m.group(2)
                instanced = self.transformer.instantiate_component(comp_name, attrs)
                return instanced.replace('<slot></slot>', slot_content).replace('<slot/>', slot_content)
            source = pattern.sub(replace_instance, source)
            
            # Forme sans slot: <ComponentName />
            pattern_closed = re.compile(
                fr'<{comp_name}\s*((?:[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{{[^}}]*}}|[\w./:-]+))?\s*)*)/>',
                re.DOTALL | re.IGNORECASE
            )
            def replace_instance_closed(m):
                attrs = parse_inline_attrs(m.group(1))
                return self.transformer.instantiate_component(comp_name, attrs)
            source = pattern_closed.sub(replace_instance_closed, source)
        return source

    # ── 6. Blocs if ──────────────────────────────────────────────────────────

    def _process_if_blocks(self, source: str) -> str:
        # Regex robuste : supporte > dans les accolades de condition
        pattern = re.compile(
            r'<if\s+((?:[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?\s*)*)>'
            r'(.*?)</if>',
            re.DOTALL | re.IGNORECASE
        )
        def replace_if(m):
            attrs = parse_inline_attrs(m.group(1))
            cond = str(attrs.get('condition', ''))
            content = m.group(2).strip()
            remaining = source[m.end():]
            elseif_blocks, else_content = [], ''
            ei_re = re.compile(r'^\s*<elseif\s+([^>]+)>(.*?)</elseif>', re.DOTALL | re.IGNORECASE)
            while True:
                ei = ei_re.match(remaining)
                if not ei: break
                ei_attrs = parse_inline_attrs(ei.group(1))
                elseif_blocks.append((str(ei_attrs.get('condition', '')), ei.group(2).strip()))
                remaining = remaining[ei.end():]
            el = re.match(r'^\s*<else>(.*?)</else>', remaining, re.DOTALL | re.IGNORECASE)
            if el: else_content = el.group(1).strip()
            return self.transformer.transform_if_block(cond, content, elseif_blocks, else_content)
        source = pattern.sub(replace_if, source)
        source = re.sub(r'<elseif\s+[^>]+>.*?</elseif>', '', source, flags=re.DOTALL | re.IGNORECASE)
        source = re.sub(r'<else>.*?</else>', '', source, flags=re.DOTALL | re.IGNORECASE)
        return source

    # ── 7. Boucles each ──────────────────────────────────────────────────────

    def _process_each_blocks(self, source: str) -> str:
        # Regex robuste : capture l'intégralité des attributs y compris les filtres avec >
        pattern = re.compile(
            r'<each\s+((?:[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?\s*)*)>'
            r'(.*?)</each>',
            re.DOTALL | re.IGNORECASE
        )
        def replace_each(m):
            attrs = parse_inline_attrs(m.group(1))
            in_var = str(attrs.get('in', '')).strip('{}')
            as_name = str(attrs.get('as', 'item'))
            index_name = attrs.get('index')
            filter_expr = attrs.get('filter')
            tag_name = str(attrs.get('tag', 'div'))
            return self.transformer.transform_each(
                in_var, as_name, index_name, filter_expr, m.group(2).strip(), tag_name
            )
        source = pattern.sub(replace_each, source)
        source = re.sub(
            r'<empty\s+for=["\']#?(\w+)["\'][^>]*>(.*?)</empty>',
            lambda m: f'<div data-nhtml-empty="{m.group(1)}" style="display:none">{m.group(2).strip()}</div>',
            source, flags=re.DOTALL | re.IGNORECASE
        )
        return source

    # ── 8. Assemblage ─────────────────────────────────────────────────────────

    def _assemble(self, body: str) -> str:
        # Runtime et Manifest JS TOUJOURS en premier
        state_js = self.transformer.baseSetup()
        all_css = '\n'.join(self.transformer.css_blocks)

        if '<body' in body:
            body = body.replace('</body>', f'{state_js}\n</body>')
            if '<head>' in body:
                body = body.replace('</head>', f'<style>\n{all_css}\n</style>\n</head>')
            return body

        return f'{body}\n<style>{all_css}</style>\n{state_js}'
