use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error, warn};
use redis::AsyncCommands;
use futures_util::StreamExt;

pub async fn start_cluster_bridge(
    gateway_id: String,
    redis_url: String,
    tx_app_broadcast: broadcast::Sender<Arc<Vec<u8>>>,
) {
    info!("⚡ Cluster: Démarrage du bridge Redis sur {} (GID: {})", redis_url, gateway_id);
    
    let client = match redis::Client::open(redis_url.clone()) {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Cluster: Échec de connexion Redis : {}", e);
            return;
        }
    };

    // 1. Task de Publication (Local -> Redis)
    let mut rx_local = tx_app_broadcast.subscribe();
    let pub_client = client.clone();
    let my_gid = gateway_id.clone();
    tokio::spawn(async move {
        let mut pub_conn = match pub_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("❌ Cluster: Connexion Publication multiplexée échouée : {}", e);
                return;
            }
        };
        
        while let Ok(msg_arc) = rx_local.recv().await {
            let msg = &*msg_arc;
            if msg.len() < 2 { continue; }
            
            let gid_len = msg[1] as usize;
            if msg.len() < 2 + gid_len { continue; }
            let sender_gid = String::from_utf8_lossy(&msg[2..2+gid_len]);
            
            if sender_gid == my_gid {
                let _: Result<(), _> = pub_conn.publish("nhtml:broadcast", msg).await;
            }
        }
    });

    // 2. Task de Souscription (Redis -> Local)
    let mut pubsub = match client.get_async_pubsub().await {
        Ok(ps) => ps,
        Err(e) => {
            error!("❌ Cluster: Impossible d'obtenir un client PubSub Async : {}", e);
            return;
        }
    };
    
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

        if payload.len() < 2 { continue; }
        let gid_len = payload[1] as usize;
        if payload.len() < 2 + gid_len { continue; }
        let sender_gid = String::from_utf8_lossy(&payload[2..2+gid_len]);

        // Si ça vient d'une AUTRE gateway, on relaie en local
        if sender_gid != gateway_id {
            let _ = tx_app_broadcast.send(Arc::new(payload));
        }
    }
}
