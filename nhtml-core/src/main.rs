use std::env;
use std::fs;
use nhtml_core;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: nhtml <fichier.nhtml> [sortie_base]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    
    // Détermination dynamique du nom de sortie
    let output_base = if args.len() > 2 {
        args[2].clone()
    } else {
        input_path.replace(".nhtml", "")
    };
    
    let source = fs::read_to_string(input_path)
        .expect("Impossible de lire le fichier source");

    // Appel du moteur de compilation natif
    let result = nhtml_core::compile(&source);

    // Écriture des deux fichiers de sortie (HTML + Manifest)
    fs::write(format!("{}.html", output_base), &result.html)
        .expect("Erreur lors de l'écriture du fichier HTML");
    
    fs::write(format!("{}.json", output_base), &result.manifest_json)
        .expect("Erreur lors de l'écriture du fichier JSON (manifest)");

    println!("✓ {} -> {}.html + {}.json", 
        input_path, output_base, output_base);
}
