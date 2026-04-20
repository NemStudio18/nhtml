use std::env;
use std::fs;
use nhtml_core;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: nhtml [--render | --cgi] <fichier.nhtml> [sortie_base]");
        std::process::exit(1);
    }

    let mut is_render = false;
    let mut is_cgi = false;
    let mut input_path = String::new();
    let mut output_base = String::new();

    // Parsing simple des arguments
    for (i, arg) in args.iter().enumerate() {
        if i == 0 { continue; }
        if arg == "--render" { is_render = true; }
        else if arg == "--cgi" { is_cgi = true; }
        else if input_path.is_empty() { input_path = arg.clone(); }
        else if output_base.is_empty() { output_base = arg.clone(); }
    }

    // Gestion du mode CGI (auto-détection du fichier via environnement)
    if is_cgi {
        if let Ok(filename) = env::var("SCRIPT_FILENAME") {
            input_path = filename;
        }
        // Headers HTTP requis pour CGI
        println!("Content-Type: text/html; charset=utf-8");
        println!(); // Séparateur header/body
    }

    if input_path.is_empty() {
        eprintln!("Erreur: Aucun fichier d'entrée spécifié.");
        std::process::exit(1);
    }
    
    let source = fs::read_to_string(&input_path)
        .expect("Impossible de lire le fichier source");

    // Appel du moteur de compilation natif
    let result = nhtml_core::compile(&source);

    if is_render || is_cgi {
        // En mode direct, on affiche simplement l'HTML produit
        println!("{}", result.html);
    } else {
        // Mode classique : écriture de fichiers
        let final_base = if !output_base.is_empty() {
            output_base
        } else {
            input_path.replace(".nhtml", "")
        };
        
        fs::write(format!("{}.html", final_base), &result.html)
            .expect("Erreur lors de l'écriture du fichier HTML");
        
        fs::write(format!("{}.json", final_base), &result.manifest_json)
            .expect("Erreur lors de l'écriture du fichier JSON (manifest)");

        eprintln!("✓ {} -> {}.html + {}.json", input_path, final_base, final_base);
    }
}
