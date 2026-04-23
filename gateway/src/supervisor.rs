use tokio::process::Command;
use tokio::signal;

pub async fn start_php_server(port: u16) {
    println!("⚙️ Supervisor: Lancement de PHP-in-a-Box sur le port {}...", port);

    let mut child = Command::new("php")
        .arg("-S")
        .arg(format!("127.0.0.1:{}", port))
        .spawn()
        .expect("❌ Échec du lancement de PHP. Vérifiez que PHP est installé et dans le PATH.");

    println!("✅ Supervisor: Serveur PHP opérationnel.");

    // Gestion propre de l'arrêt (Ctrl+C) pour éviter les processus zombies
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Erreur lors de l'écoute du signal Ctrl+C");
        println!("\n🛑 Supervisor: Signal d'arrêt reçu, fermeture du serveur PHP...");
        let _ = child.kill().await;
        std::process::exit(0);
    });
}
