use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error, warn};
use redis::AsyncCommands;
use redis::AsyncIter;

pub async fn start_cluster_bridge(
    redis_url: String,
    tx_app_broadcast: broadcast::Sender<Arc<Vec<u8>>>,
) {
    info!("⚡ Cluster: Démarrage du bridge Redis sur {}...", redis_url);
    
    let client = match redis::Client::open(redis_url.clone()) {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Cluster: Échec de connexion Redis : {}", e);
            return;
        }
    };

    let mut conn = match client.get_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Cluster: Impossible d'obtenir une connexion Redis : {}", e);
            return;
        }
    };

    // 1. Task de Publication (Local -> Redis)
    let mut rx_local = tx_app_broadcast.subscribe();
    let pub_client = client.clone();
    tokio::spawn(async move {
        let mut pub_conn = pub_client.get_async_connection().await.unwrap();
        while let Ok(msg) = rx_local.recv().await {
            // Éviter de republier ce qui vient déjà de Redis (marquage interne ou filtrage)
            // Pour l'instant, on publie tout, mais il faudra un ID de Gateway pour filtrer
            let _: Result<(), _> = pub_conn.publish("nhtml:broadcast", msg.as_slice()).await;
        }
    });

    // 2. Task de Souscription (Redis -> Local)
    let mut pubsub = client.get_async_connection().await.unwrap().into_pubsub();
    if let Err(e) = pubsub.subscribe("nhtml:broadcast").await {
        error!("❌ Cluster: Échec de souscription Redis : {}", e);
        return;
    }

    info!("⚡ Cluster: Connecté et synchronisé via Redis.");
    
    let mut stream = pubsub.on_message();
    while let Some(msg) = stream.next().await {
        let payload: Vec<u8> = match msg.get_payload() {
            Ok(p) => p,
            Err(_) => continue,
        };
        
        // Relayer le message aux clients locaux
        let _ = tx_app_broadcast.send(Arc::new(payload));
    }
}
