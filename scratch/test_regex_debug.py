import re

def parse_inline_attrs(attrs_str: str) -> dict:
    attrs = {}
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

test_str = '<var complex_var=\'{"items": [{"label":"<Danger>"}, {"label": "Safer"}], "active": true, "regex": "/[a-z]+/g"}\'>'
# Simulate what _process_vars does
inner = test_str[5:-1] # complex_var='...'
print(f"Inner: {inner}")
res = parse_inline_attrs(inner)
print(f"Result: {res}")
