use notify::{Watcher, RecursiveMode, Event};
use std::path::Path;
use tokio::sync::broadcast;
use std::time::Duration;

#[allow(dead_code)]
pub fn start_watcher(tx: broadcast::Sender<()>) {
    println!("👀 Watcher: Démarrage de la surveillance des fichiers locaux...");

    std::thread::spawn(move || {
        let (tx_notify, rx_notify) = std::sync::mpsc::channel();
        
        let mut watcher = notify::recommended_watcher(tx_notify)
            .expect("❌ Échec de création du Watcher");

        // Surveille le dossier courant récursivement
        watcher.watch(Path::new("."), RecursiveMode::Recursive)
            .expect("❌ Échec de la surveillance du dossier");

        // Extensions sources qui déclenchent le Hot Reload
        const SOURCE_EXTENSIONS: &[&str] = &["nhtml", "php", "js", "css", "html"];

        // Boucle de réception des événements du filesystem
        for res in rx_notify {
            match res {
                Ok(Event { kind: _, paths, .. }) => {
                    let path_str = paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>().join("");
                    let filename = paths.get(0).and_then(|p| p.file_name()).and_then(|f| f.to_str()).unwrap_or("");

                    // Ignorer les fichiers système / runtime / état / SQLite
                    if path_str.contains(".git") 
                        || path_str.contains("target") 
                        || path_str.contains(".db")
                        || path_str.contains("-journal")
                        || path_str.contains("-wal")
                        || path_str.contains("-shm")
                        || path_str.contains(".state")
                        || path_str.contains(".log")
                        || path_str.contains(".lock")
                        || path_str.contains(".tmp")
                    {
                        continue;
                    }

                    // Déclencher uniquement sur les fichiers sources connus
                    let is_source = paths.iter().any(|p| {
                        p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| SOURCE_EXTENSIONS.contains(&ext))
                            .unwrap_or(false)
                    });

                    if !is_source {
                        continue;
                    }
                    
                    println!("🔄 Hot Reload: Modification détectée sur [{}]", filename);
                    println!("🔥 Envoi du signal de HOT RELOAD...");
                    let _ = tx.send(());
                    
                    // Debounce : ignore les événements suivants pendant 500ms
                    std::thread::sleep(Duration::from_millis(500));
                },
                Err(e) => println!("Erreur Watcher: {:?}", e),
            }
        }
    });
}
