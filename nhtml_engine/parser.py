import re
import os
import json
from nhtml_engine.utils import dict_to_html_attrs, parse_inline_attrs
from nhtml_engine.transformer import NhtmlTransformer

class NhtmlParser:
    """
    Parser principal : lit un fichier .nhtml et produit HTML + CSS + JS.
    """

    # Balises Nhtml spéciales qui ne passent pas dans le HTML final
    NHTML_TAGS = {"nhtml", "var", "component", "import", "each", "empty",
                  "if", "elseif", "else", "props", "prop", "slot"}

    def __init__(self):
        self.transformer = NhtmlTransformer()

    def parse(self, source: str) -> str:
        """Point d'entrée principal. Retourne le HTML final."""

        # 1. Retirer la déclaration <!nhtml ...>
        source = re.sub(r'<!nhtml[^>]*>', '', source).strip()

        # 2. Traiter les balises <var>
        source = self._process_vars(source)

        # 3. Traiter les attributs Nhtml sur les balises standards
        # On le fait avant les blocs pour que l'intérieur des if/each soit traité
        source = self._process_nhtml_attrs(source)

        # 4. Traiter le titre spécifiquement (réactivité sans span)
        source = self._process_title_tag(source)

        # 5. Traiter l'interpolation dans le texte brut
        source = self._process_text_nodes(source)

        # 5. Traiter les composants <component>
        source = self._process_components(source)

        # 6. Traiter les blocs <if><elseif><else>
        source = self._process_if_blocks(source)

        # 7. Traiter les boucles <each>
        source = self._process_each_blocks(source)

        # 8. Assembler le document final
        return self._assemble(source)

    # ── ÉTAPE 1 : Variables ──────────────────────────────────────────────────

    def _process_vars(self, source: str) -> str:
        """
        Extrait les <var name=value> et les retire du HTML.
        Gère les valeurs complexes : tableaux JSON, objets, strings, booléens.
        """
        # Pattern qui capture nom=valeur en gérant les [ ] et { } imbriqués
        var_pattern = re.compile(r'<var\s+(.*?)/?>', re.IGNORECASE | re.DOTALL)

        def replace_var(m):
            raw = m.group(1).strip()
            is_persistent = "persist" in raw.lower().split()
            
            # Parser nom=valeur avec support des tableaux/objets JSON
            pair_pattern = re.compile(
                r'(\w+)\s*=\s*'
                r'('
                r'\[.*?\]'      # tableau JSON
                r'|\{.*?\}'     # objet JSON
                r'|"[^"]*"'     # string double quotes
                r"|'[^']*'"     # string simple quotes
                r'|true|false'  # booléens
                r'|-?\d+\.?\d*' # nombres
                r'|\w+'         # mots simples
                r')',
                re.DOTALL
            )
            for pm in pair_pattern.finditer(raw):
                name = pm.group(1)
                raw_value = pm.group(2).strip()
                # Convertir la valeur
                if raw_value == "true":
                    value = True
                elif raw_value == "false":
                    value = False
                elif raw_value.startswith('"') or raw_value.startswith("'"):
                    value = raw_value[1:-1]
                else:
                    try:
                        value = json.loads(raw_value)
                    except Exception:
                        value = raw_value
                self.transformer.vars[name] = value
                if is_persistent:
                    self.transformer.persistent_vars.add(name)
            return ""

        source = var_pattern.sub(replace_var, source)
        if self.transformer.vars:
            self.transformer.js_blocks = [self.transformer._generate_reactivity_system()]
        return source

    # ── ÉTAPE 2 : Composants ─────────────────────────────────────────────────

    def _process_components(self, source: str) -> str:
        """Extrait les définitions <component name='...'>...</component>."""
        pattern = re.compile(
            r'<component\s+name=["\']([^"\']+)["\'][^>]*>(.*?)</component>',
            re.DOTALL | re.IGNORECASE
        )

        def replace_component(m):
            name = m.group(1)
            content = m.group(2).strip()
            # Extraire les props
            props = self._extract_props(content)
            content = re.sub(r'<props>.*?</props>', '', content, flags=re.DOTALL).strip()
            self.transformer.register_component(name, content, props)
            return ""

        return pattern.sub(replace_component, source)

    def _extract_props(self, content: str) -> list:
        """Extrait les <prop> d'un bloc <props>.</prop>"""
        props = []
        props_match = re.search(r'<props>(.*?)</props>', content, re.DOTALL)
        if props_match:
            prop_pattern = re.compile(r'<prop\s+([^>]+?)/?>', re.IGNORECASE)
            for pm in prop_pattern.finditer(props_match.group(1)):
                props.append(parse_inline_attrs(pm.group(1)))
        return props

    # ── ÉTAPE 3 : Imports ────────────────────────────────────────────────────

    def _process_imports(self, source: str) -> str:
        """Traite les <import src='...' as='...'/>."""
        pattern = re.compile(r'<import\s+([^>]+?)/?>', re.IGNORECASE)

        def replace_import(m):
            attrs = parse_inline_attrs(m.group(1))
            src = attrs.get("src", "")
            as_name = attrs.get("as", "")
            if src and as_name and os.path.exists(src):
                with open(src, encoding="utf-8") as f:
                    comp_source = f.read()
                # Parser le composant importé
                sub_parser = NhtmlParser()
                inner = re.search(r'<component>(.*?)</component>', comp_source, re.DOTALL)
                if inner:
                    props = sub_parser._extract_props(inner.group(1))
                    content = re.sub(r'<props>.*?</props>', '', inner.group(1), flags=re.DOTALL).strip()
                    self.transformer.register_component(as_name, content, props)
            return ""

        return pattern.sub(replace_import, source)

    # ── ÉTAPE 4 : Blocs if/elseif/else ──────────────────────────────────────

    def _process_if_blocks(self, source: str) -> str:
        """
        Transforme les blocs <if condition='...'> ... </if> <elseif> <else>
        """
        # On cherche d'abord les blocs <if>...</if> proprement
        pattern = re.compile(r'<if\s+([^>]+)>(.*?)</if>', re.DOTALL | re.IGNORECASE)

        def replace_if(m):
            attrs = parse_inline_attrs(m.group(1))
            condition = attrs.get("condition", "")
            if_content = m.group(2).strip()
            
            # On cherche les blocs elseif/else qui suivent immédiatement le </if>
            remaining = source[m.end():]
            
            elseif_blocks = []
            else_content = ""
            
            # Matcher les elseif successifs
            ei_pattern = re.compile(r'^\s*<elseif\s+([^>]+)>(.*?)</elseif>', re.DOTALL | re.IGNORECASE)
            while True:
                ei_match = ei_pattern.match(remaining)
                if not ei_match: break
                ei_attrs = parse_inline_attrs(ei_match.group(1))
                elseif_blocks.append((ei_attrs.get("condition", ""), ei_match.group(2).strip()))
                remaining = remaining[ei_match.end():]

            # Matcher le else final
            el_pattern = re.compile(r'^\s*<else>(.*?)</else>', re.DOTALL | re.IGNORECASE)
            el_match = el_pattern.match(remaining)
            if el_match:
                else_content = el_match.group(1).strip()
                remaining = remaining[el_match.end():]

            return self.transformer.transform_if_block(
                condition, if_content, elseif_blocks, else_content
            )

        source = pattern.sub(replace_if, source)
        source = re.sub(r'<elseif\s+[^>]+>.*?</elseif>', '', source, flags=re.DOTALL | re.IGNORECASE)
        source = re.sub(r'<else>.*?</else>', '', source, flags=re.DOTALL | re.IGNORECASE)
        return source

    # ── ÉTAPE 5 : Boucles <each> ─────────────────────────────────────────────

    def _process_each_blocks(self, source: str) -> str:
        """Transforme les blocs <each in='...' as='...'> ... </each>"""
        # Regex ROBUSTE pour <each ...> : on ne s'arrête pas au premier > s'il est dans une string ou expression
        pattern = re.compile(
            r'<each((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)\s*>(.*?)</each>',
            re.DOTALL | re.IGNORECASE
        )

        def replace_each(m):
            attrs = parse_inline_attrs(m.group(1))
            in_var = attrs.get("in", "")
            as_name = attrs.get("as", "item")
            index_name = attrs.get("index", None)
            filter_expr = attrs.get("filter", None)
            tag_name = attrs.get("tag", "div")
            template = m.group(2).strip()
            return self.transformer.transform_each(in_var, as_name, index_name, filter_expr, template, tag_name)

        # Traiter aussi <empty for="#id">
        source = pattern.sub(replace_each, source)
        source = re.sub(
            r'<empty\s+for=["\']#?(\w+)["\'][^>]*>(.*?)</empty>',
            lambda m: f'<div data-nhtml-empty="{m.group(1)}" style="display:none">{m.group(2).strip()}</div>',
            source, flags=re.DOTALL | re.IGNORECASE
        )
        return source

    # ── ÉTAPE 6 : Attributs Nhtml sur balises standards ───────────────────────

    def _process_nhtml_attrs(self, source: str) -> str:
        """
        Traite les attributs spéciaux (text, html, bind:, for:each, visible) 
        et gère aussi le scoping des variables dans les attributs normaux.
        """
        # Regex vérifiée par test_regex.py : robuste aux > dans les attributs
        tag_pattern = re.compile(r'<(\w+)((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)\s*(/?)>', re.DOTALL)

        def replace_tag(m):
            tag = m.group(1)
            attrs_str = m.group(2).strip()
            self_close = m.group(3) == "/"           # Ignorer les balises purement Nhtml déjà traitées
            if tag in self.NHTML_TAGS:
                return m.group(0)

            attrs = parse_inline_attrs(attrs_str)

            # Vérifier si c'est un composant enregistré
            if tag in self.transformer.components:
                return self.transformer.instantiate_component(tag, attrs)

            # Transformer les événements on:
            attrs = self.transformer.transform_events(attrs)

            # Transformer if= attribut simple
            attrs = self.transformer.transform_if_attr(attrs)

            # Transformer for:each= simple
            if "for:each" in attrs:
                return self.transformer.transform_for_each(tag, attrs)

            # Transformer bind:value=
            if any(k.startswith("bind:") for k in attrs):
                return self.transformer.transform_input(tag, attrs)

            # Extraire text=, html= et label=
            text_val = attrs.pop("text", None)
            html_val = attrs.pop("html", None)
            label_val = attrs.pop("label", None)
            visible_val = attrs.pop("visible", None)

            # Gestion visible=
            # Gestion visible= (data-nhtml-if)
            if visible_val is not None:
                var_name = str(visible_val)
                # On retire les accolades pour rendre l'expression compatible JS
                var_name = var_name.replace('{', '').replace('}', '')
                attrs["data-nhtml-if"] = var_name
                # On force display:none par défaut pour éviter le flickering
                if "style" not in attrs: attrs["style"] = "display:none"
                elif "display:none" not in attrs["style"]: attrs["style"] += ";display:none"

            # On vérifie aussi si un data-nhtml-if a été ajouté via transform_if_attr
            # et on applique le même principe de masquage par défaut.
            if "data-nhtml-if" in attrs:
                if "style" not in attrs: attrs["style"] = "display:none"
                elif "display:none" not in attrs["style"]: attrs["style"] += ";display:none"

            # Attributs dynamiques (avec {})
            dynamic_attrs = {}
            for k, v in list(attrs.items()):
                if isinstance(v, str) and "{" in v and k not in ("data-nhtml-if", "data-nhtml-text", "data-nhtml-html"):
                    dynamic_attrs[k] = v
                    del attrs[k]

            html_attrs = dict_to_html_attrs(attrs)
            parts = [f"<{tag}"]
            if html_attrs:
                parts.append(f" {html_attrs}")

            if text_val:
                parts.append(f' data-nhtml-text="{text_val}"')
            
            if html_val:
                parts.append(f' data-nhtml-html="{html_val}"')

            if dynamic_attrs:
                escaped = json.dumps(dynamic_attrs).replace('"', '&quot;')
                parts.append(f' data-nhtml-attrs="{escaped}"')

            # display_str : seulement si label_val est une string (pas None ou bool)
            if isinstance(label_val, str):
                display_str = label_val
            else:
                display_str = ""
            
            # Auto-détection : si le contenu fixe contient des { }, on l'ajoute en data-nhtml-text
            if not text_val and isinstance(label_val, str) and "{" in label_val:
                parts.append(f' data-nhtml-text="{label_val}"')

            if self_close or tag in ("input", "br", "hr", "img", "meta", "link"):
                parts.append(f">{display_str}</{tag}>" if display_str else "/>")
            else:
                parts.append(f">{display_str}")

            return "".join(str(p) for p in parts)

        return tag_pattern.sub(replace_tag, source)

    def _process_title_tag(self, source: str) -> str:
        """
        Gère le cas particulier de la balise <title> pour qu'elle soit réactive
        sans injecter de <span> à l'intérieur.
        """
        def replace_title(m):
            attrs_str = m.group(1)
            content = m.group(2).strip()
            
            # Si le contenu contient une expression {expr}
            if "{" in content:
                attrs = parse_inline_attrs(attrs_str)
                # On ajoute le contenu tel quel dans data-nhtml-text pour le système JS
                attrs["data-nhtml-text"] = content
                html_attrs = dict_to_html_attrs(attrs)
                return f"<title {html_attrs}>{content}</title>"
            return m.group(0)

        return re.sub(r'<title([^>]*)>(.*?)</title>', replace_title, source, flags=re.DOTALL | re.IGNORECASE)

    # ── ÉTAPE 5 : Interpolation texte libre ─────────────────────────────────

    def _process_text_nodes(self, source: str) -> str:
        """
        Détecte les {expression} dans le texte libre et les enveloppe dans des spans réactifs.
        Ignore ce qui se trouve dans les balises et les blocs protégés.
        """
        # On protège les blocs script, style, title
        protected = []
        def protect(m):
            protected.append(m.group(0))
            return f"<!--__NHTML_PROTECTED_{len(protected)-1}__-->"

        source = re.sub(r'<(script|style|title)[^>]*>.*?</\1>', protect, source, flags=re.DOTALL | re.IGNORECASE)

        # Protection des tags par la même regex ROBUSTE pour ne pas couper sur un > interne
        tag_pattern = re.compile(r'<[\w.-]+(?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*\s*/?>', re.DOTALL)
        source = tag_pattern.sub(protect, source)

        # On remplace {expression} par un span réactif
        source = re.sub(r'\{([^{}\n\r]+?)\}', r'<span data-nhtml-text="{\1}">{\1}</span>', source)

        # Restauration
        def restore(m):
            idx = int(m.group(1))
            return protected[idx]

        while "<!--__NHTML_PROTECTED_" in source:
            source = re.sub(r'<!--__NHTML_PROTECTED_(\d+)__-->', restore, source)
        
        return source

    # ── ASSEMBLAGE FINAL ─────────────────────────────────────────────────────

    def _assemble(self, body: str) -> str:
        """Assemble le document HTML final avec JS et CSS injectés."""

        # Récupérer les blocs <style> et <script> du body
        styles = re.findall(r'<style[^>]*>(.*?)</style>', body, re.DOTALL)
        body = re.sub(r'<style[^>]*>.*?</style>', '', body, flags=re.DOTALL)

        # Séparer les <script src="..."> (externes, à conserver tels quels)
        external_scripts = re.findall(r'<script\s+[^>]*src=[^>]+>\s*</script>', body, re.DOTALL | re.IGNORECASE)
        body = re.sub(r'<script\s+[^>]*src=[^>]+>\s*</script>', '', body, flags=re.DOTALL | re.IGNORECASE)

        # Récupérer les scripts inline (sans src)
        scripts = re.findall(r'<script(?:\s(?!.*?\bsrc\b)[^>]*)?>(?!\s*</script>)(.*?)</script>', body, re.DOTALL | re.IGNORECASE)
        body = re.sub(r'<script(?:\s(?!.*?\bsrc\b)[^>]*)?>(?!\s*</script>).*?</script>', '', body, flags=re.DOTALL | re.IGNORECASE)

        # CSS final
        all_css = "\n".join(styles + self.transformer.css_blocks)

        # JS final : système de réactivité + blocs générés + scripts utilisateur
        all_js = "\n".join(self.transformer.js_blocks + scripts)

        # Récupérer le <head> existant si présent
        head_match = re.search(r'<head[^>]*>(.*?)</head>', body, re.DOTALL)
        head_content = head_match.group(1) if head_match else "<title>Nhtml App</title>"
        body = re.sub(r'<head[^>]*>.*?</head>', '', body, flags=re.DOTALL).strip()

        # Extraire les scripts externes du head (ex: pell.min.js) et les conserver
        head_external = re.findall(r'<script\s+[^>]*src=[^>]+>\s*</script>', head_content, re.IGNORECASE)
        head_content = re.sub(r'<script\s+[^>]*src=[^>]+>\s*</script>', '', head_content, flags=re.IGNORECASE)
        external_scripts = head_external + external_scripts

        # Réinjecter les scripts externes dans la section head
        external_scripts_html = "\n    ".join(external_scripts)

        # Extraire le contenu du <body> si présent, sinon garder tel quel
        body_match = re.search(r'<body[^>]*>(.*?)</body>', body, re.DOTALL)
        if body_match:
            body = body_match.group(1).strip()
        else:
            body = re.sub(r'</?body[^>]*>', '', body).strip()

        # Nettoyer les lignes vides multiples
        body = re.sub(r'\n{3,}', '\n\n', body).strip()

        return f"""<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {head_content}
    {external_scripts_html}
    <style>
    {all_css}
    </style>
</head>
<body>
{body}
<script>
{all_js}
</script>
</body>
</html>"""
