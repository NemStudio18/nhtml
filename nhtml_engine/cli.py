import sys
import os
import time
import logging
from nhtml_engine.parser import NhtmlParser
from nhtml_engine.transformer import NHTML_RUNTIME_JS

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

def compile_file(input_file, output_file, runtime_mode='inline'):
    try:
        with open(input_file, encoding="utf-8") as f:
            source = f.read()
        parser = NhtmlParser(runtime_mode=runtime_mode)
        result = parser.parse(source)
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(result)
        logging.info(f"Compiled: {input_file} -> {output_file}")
    except Exception as e:
        logging.error(f"Erreur de compilation sur {input_file}: {str(e)}")

def build_dir(input_dir, output_dir, runtime_mode='inline'):
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
                compile_file(input_path, output_path, runtime_mode)
                compiled_count += 1
    return compiled_count

def watch_dir(input_dir, output_dir, runtime_mode='inline'):
    logging.info(f"Watching directory '{input_dir}' for changes... (mode: {runtime_mode})")
    build_dir(input_dir, output_dir, runtime_mode)
    
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
                build_dir(input_dir, output_dir, runtime_mode)
            
            time.sleep(1)
    except KeyboardInterrupt:
        logging.info("Arrêt du mode watch.")

def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python nhtml.py <fichier.nhtml> [sortie.html]")
        print("  python nhtml.py build <input_dir> <output_dir> [--runtime=inline|cdn|local.js|none] [--no-runtime]")
        print("  python nhtml.py watch <input_dir> <output_dir> [--runtime=...]")
        print("  python nhtml.py export-runtime <sortie.js>")
        sys.exit(1)

    args = sys.argv[1:]
    command = args.pop(0)

    # Parsing arguments optionnels
    runtime_mode = 'inline'
    pure_args = []
    
    for arg in args:
        if arg.startswith('--runtime='):
            runtime_mode = arg.split('=', 1)[1]
        elif arg == '--no-runtime':
            runtime_mode = 'none'
        else:
            pure_args.append(arg)

    if command == "export-runtime":
        if len(pure_args) < 1:
            print("Usage: python nhtml.py export-runtime <sortie.js>")
            sys.exit(1)
        out_file = pure_args[0]
        os.makedirs(os.path.dirname(os.path.abspath(out_file)), exist_ok=True)
        with open(out_file, 'w', encoding='utf-8') as f:
            f.write(NHTML_RUNTIME_JS.strip())
        print(f"[OK] Runtime exporté avec succès vers {out_file}")
        sys.exit(0)

    if command == "build" or command == "watch":
        if len(pure_args) < 2:
            print(f"Usage: python nhtml.py {command} <input_dir> <output_dir>")
            sys.exit(1)
        input_dir = pure_args[0]
        output_dir = pure_args[1]
        if not os.path.exists(input_dir):
            print(f"Erreur : répertoire '{input_dir}' introuvable.")
            sys.exit(1)
            
        if command == "build":
            c = build_dir(input_dir, output_dir, runtime_mode)
            print(f"Build completed: {c} files compiled (mode: {runtime_mode}).")
        else:
            watch_dir(input_dir, output_dir, runtime_mode)
    else:
        # Mode classique simple fichier
        input_file = command
        output_file = pure_args[0] if len(pure_args) > 0 else input_file.replace(".nhtml", ".html")
        if not os.path.exists(input_file):
            print(f"Erreur : fichier '{input_file}' introuvable.")
            sys.exit(1)
        compile_file(input_file, output_file, runtime_mode)

