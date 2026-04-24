use tokio::process::Command;
use tokio::signal;
use std::path::Path;

pub async fn start_php_server(port: u16) {
    // --- Algorithme de détection du binaire ---
    let php_bin = if Path::new("./php.exe").exists() {
        "./php.exe".to_string()
    } else if Path::new("./php/php.exe").exists() {
        "./php/php.exe".to_string()
    } else if Path::new("./bin/php.exe").exists() {
        "./bin/php.exe".to_string()
    } else {
        "php".to_string() // Fallback PATH
    };

    println!("⚙️ Supervisor: Tentative de lancement avec : {}", php_bin);

    let child_res = Command::new(&php_bin)
        .arg("-S")
        .arg(format!("127.0.0.1:{}", port))
        .arg("router.php")
        .spawn();

    let mut child = match child_res {
        Ok(c) => {
            println!("✅ Supervisor: Serveur PHP opérationnel sur le port {}.", port);
            c
        },
        Err(e) => {
            eprintln!("❌ Supervisor: Impossible de lancer PHP ({}).", e);
            eprintln!("👉 Installez PHP ou placez un dossier 'php/' contenant le binaire à côté du Gateway.");
            return;
        }
    };

    // Gestion propre de l'arrêt (Ctrl+C) pour éviter les processus zombies
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Erreur lors de l'écoute du signal Ctrl+C");
        println!("\n🛑 Supervisor: Signal d'arrêt reçu, fermeture du serveur PHP...");
        let _ = child.kill().await;
        std::process::exit(0);
    });
}
