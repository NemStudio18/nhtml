from nhtml_engine.parser import NhtmlParser

html = """
<var site_name="Test CMS">
<header>
    <h1>{site_name}</h1>
</header>
<main>
    <if condition="true">
        <p>Visible!</p>
    </if>
    <each in="{[1,2]}" as="i">
        <li>Item {i}</li>
    </each>
</main>
"""

parser = NhtmlParser()
result = parser.parse(html)
# Force UTF-8 output for Windows terminals
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
print(result)
