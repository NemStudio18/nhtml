from nhtml_engine.parser import NhtmlParser
import re

html = '<h2 if="!{current_category}"><i class="fas fa-feather-alt"></i> Derniers articles</h2>'

parser = NhtmlParser()
print("1. Original:", html)

# Simuler les étapes
# Step 3: Attributes
source = parser._process_nhtml_attrs(html)
print("2. After Attributes:", source)

# Step 5: Text nodes
source = parser._process_text_nodes(source)
print("3. After Text Nodes:", source)
