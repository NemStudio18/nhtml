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
        r'|(\{(?:[^{}]|\{[^{}]*\})*\})'  # valeur entre accolades (supporte 1 niveau d'imbrication)
        r'|([^>\s"\'=]+)'       # valeur sans quotes
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

def js_to_opcodes(js_str: str) -> list:
    """
    Parse une string JavaScript rudimentaire ("counter += 1; alert('Hello')")
    en un tableau d'opcodes déclaratifs independant.
    """
    ops = []
    commands = [c.strip() for c in js_str.split(';') if c.strip()]
    
    for cmd in commands:
        cmd_trim = cmd.strip()
        if cmd_trim.endswith("++"):
            target = cmd_trim[:-2].strip()
            ops.append({"op": "increment", "target": target, "value": 1})
            continue
            
        if cmd_trim.endswith("--"):
            target = cmd_trim[:-2].strip()
            ops.append({"op": "increment", "target": target, "value": -1})
            continue
            
        m_inc = re.match(r'^([\w.-]+)\s*(\+|-)=\s*(.+)$', cmd)
        if m_inc:
            target, sign, value = m_inc.groups()
            op = 'increment' # On utilise toujours increment (+ ou -) pour s'aligner sur Rust
            try:
                val = int(value.strip())
            except ValueError:
                val = value.strip()
            if sign == '-': 
                if isinstance(val, int): val = -val
                else: val = f"-({val})"
            ops.append({"op": op, "target": target, "value": val})
            continue
            
        m_set = re.match(r'^([\w.-]+)\s*=\s*(.+)$', cmd)
        if m_set:
            target, value = m_set.groups()
            ops.append({"op": "set", "target": target, "value": value.strip()})
            continue
            
        m_call = re.match(r'^([\w.-]+)\s*\((.*)\)$', cmd)
        if m_call:
            fn, raw_args = m_call.groups()
            args = [a.strip() for a in raw_args.split(',') if a.strip()]
            ops.append({"op": "call", "fn": fn, "args": args})
            continue
            
        ops.append({"op": "eval", "expr": cmd})
        
    return ops
