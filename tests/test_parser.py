import os
import sys

# Ajouter le répertoire parent au path pour pouvoir importer nhtml_engine
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from nhtml_engine.parser import NhtmlParser
from nhtml_engine.transformer import NhtmlTransformer

def main():
    source_file = os.path.join(os.path.dirname(__file__), 'kitchen_sink.nhtml')
    output_file = os.path.join(os.path.dirname(__file__), 'kitchen_sink.html')

    if not os.path.exists(source_file):
        print(f"File {source_file} not found.")
        sys.exit(1)

    with open(source_file, 'r', encoding='utf-8') as f:
        content = f.read()

    print("--- DEBUT DU TEST DE COMPILATION ---")
    try:
        parser = NhtmlParser(runtime_mode='inline')
        html_output = parser.parse(content)
        
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(html_output)
            
        print("SUCCESS: Test kitchen_sink compile sans erreur.")
        print(f"Fichier de sortie : {output_file}")
    except Exception as e:
        print("ECHEC: Le parseur a crashe sur un des cas limites.")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main()
