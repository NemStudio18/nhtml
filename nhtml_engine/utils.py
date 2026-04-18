import re

def attrs_to_dict(attrs):
    """Convertit la liste de tuples attrs du HTMLParser en dict."""
    result = {}
    for name, value in attrs:
        result[name] = value if value is not None else True
    return result

def dict_to_html_attrs(attrs: dict) -> str:
    """Reconvertit un dict en string d'attributs HTML."""
    parts = []
    for k, v in attrs.items():
        if v is True:
            parts.append(k)
        else:
            parts.append(f'{k}="{v}"')
    return " ".join(parts)

def parse_inline_attrs(attrs_str: str) -> dict:
    """
    Parse une string d'attributs HTML en dict.
    Gère : attr="val", attr='val', attr=val, attr (booléen)
    """
    attrs = {}
    # Regex certifiée par test_regex.py : robuste aux > dans les attributs
    tag_pattern = re.compile(r'<(\w+)((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)\s*(/?)>', re.DOTALL)
    # Pattern qui gère les trois formes
    pattern = re.compile(
        r'([\w:.-]+)'           # nom de l'attribut
        r'(?:\s*=\s*'           # signe égal optionnel
        r'(?:"([^"]*)"'         # valeur entre double quotes
        r"|'([^']*)'"           # valeur entre simple quotes
        r'|({[^}]*})'           # valeur entre accolades
        r'|([^>\s"\'=]+)'       # valeur sans quotes (autorise !, {} etc)
        r'))?'
    )
    for m in pattern.finditer(attrs_str):
        name = m.group(1)
        value = m.group(2) or m.group(3) or m.group(4) or m.group(5)
        if value is None:
            attrs[name] = True
        else:
            attrs[name] = value
    return attrs
