import re
from nhtml_engine.utils import parse_inline_attrs

tag_pattern = re.compile(r'<(\w+)((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)\s*(/?)>', re.DOTALL)
html = '<h2 if="!{current_category}">Derniers articles</h2>'

m = tag_pattern.search(html)
tag = m.group(1)
attrs_str = m.group(2)
print(f"Tag: {tag}")
print(f"Attrs raw: '{attrs_str}'")

attrs = parse_inline_attrs(attrs_str)
print(f"Parsed attrs: {attrs}")

val = attrs.get('if')
print(f"Value of 'if': '{val}'")
clean = str(val).replace('{', '').replace('}', '')
print(f"Cleaned value: '{clean}'")
