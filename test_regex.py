import re

# La regex doit :
# 1. Ne pas s'arrêter au premier > s'il est dans des guillemets
# 2. Matcher les balises sans attributs (ex: <header>)
# 3. Matcher les balises auto-fermantes

# Regex unifiée et précise
tag_pattern = re.compile(r'<(\w+)((?:\s+[\w:.-]+(?:\s*=\s*(?:"[^"]*"|\'[^\']*\'|{[^}]*}|[\w./:-]+))?)*)\s*(/?)>', re.DOTALL)

tests = [
    '<h1 class="header">',
    '<h2 if="!{current_category}">',
    '<img src="val >" />',
    '<div visible="a > b">'
]

for t in tests:
    m = tag_pattern.match(t)
    if m:
        print(f"MATCH: {t}")
        print(f"  Tag: {m.group(1)}")
        print(f"  Attrs: '{m.group(2)}'")
        print(f"  Close: '{m.group(3)}'")
    else:
        print(f"FAIL: {t}")
