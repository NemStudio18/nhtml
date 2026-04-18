import sys
import os
import time
import logging
from nhtml_engine.parser import NhtmlParser

# Configuration du logging Python (silencieux dans le terminal par défaut sauf erreurs graves, tout va dans nhtml.log)
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S',
    handlers=[
        logging.FileHandler("nhtml.log", encoding="utf-8"),
        logging.StreamHandler(sys.stdout)
    ]
)

def compile_file(input_file, output_file):
    try:
        with open(input_file, encoding="utf-8") as f:
            source = f.read()
        parser = NhtmlParser()
        result = parser.parse(source)
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(result)
        logging.info(f"Compiled: {input_file} -> {output_file}")
    except Exception as e:
        logging.error(f"Erreur de compilation sur {input_file}: {str(e)}")

def build_dir(input_dir, output_dir):
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    
    compiled_count = 0
    for root, _, files in os.walk(input_dir):
        for file in files:
            if file.endswith('.nhtml'):
                input_path = os.path.join(root, file)
                rel_path = os.path.relpath(input_path, input_dir)
                out_name = file.replace('.nhtml', '.html')
                output_path = os.path.join(output_dir, os.path.dirname(rel_path), out_name)
                
                os.makedirs(os.path.dirname(output_path), exist_ok=True)
                compile_file(input_path, output_path)
                compiled_count += 1
    return compiled_count

def watch_dir(input_dir, output_dir):
    logging.info(f"Watching directory '{input_dir}' for changes...")
    build_dir(input_dir, output_dir)
    
    last_modified = {}
    for root, _, files in os.walk(input_dir):
        for file in files:
            if file.endswith('.nhtml'):
                filepath = os.path.join(root, file)
                last_modified[filepath] = os.path.getmtime(filepath)

    try:
        while True:
            changed = False
            for root, _, files in os.walk(input_dir):
                for file in files:
                    if file.endswith('.nhtml'):
                        filepath = os.path.join(root, file)
                        mtime = os.path.getmtime(filepath)
                        if filepath not in last_modified or last_modified[filepath] != mtime:
                            last_modified[filepath] = mtime
                            changed = True
            if changed:
                logging.info("Changes detected. Rebuilding all templates...")
                build_dir(input_dir, output_dir)
            
            time.sleep(1)
    except KeyboardInterrupt:
        logging.info("Arrêt du mode watch.")

def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python nhtml.py <fichier.nhtml> [sortie.html]")
        print("  python nhtml.py build <input_dir> <output_dir>")
        print("  python nhtml.py watch <input_dir> <output_dir>")
        sys.exit(1)

    command = sys.argv[1]

    if command == "build" or command == "watch":
        if len(sys.argv) < 4:
            print(f"Usage: python nhtml.py {command} <input_dir> <output_dir>")
            sys.exit(1)
        input_dir = sys.argv[2]
        output_dir = sys.argv[3]
        if not os.path.exists(input_dir):
            print(f"Erreur : répertoire '{input_dir}' introuvable.")
            sys.exit(1)
            
        if command == "build":
            c = build_dir(input_dir, output_dir)
            print(f"Build completed: {c} files compiled.")
        else:
            watch_dir(input_dir, output_dir)
    else:
        # Mode classique simple fichier
        input_file = command
        output_file = sys.argv[2] if len(sys.argv) > 2 else input_file.replace(".nhtml", ".html")
        if not os.path.exists(input_file):
            print(f"Erreur : fichier '{input_file}' introuvable.")
            sys.exit(1)
        compile_file(input_file, output_file)
