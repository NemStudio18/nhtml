use nhtml_gateway::session::SessionManager;

#[tokio::test]
async fn test_v0_7_1_sqlx_persistence() {
    let db_uri = "sqlite:test_sessions_v071.db";
    let db_path = std::path::Path::new("test_sessions_v071.db");
    let _ = std::fs::remove_file(db_path);
    
    let sm = SessionManager::new(db_uri).await.expect("Failed to init SM");
    
    // 1. Enregistrement d'une session
    let sid = "test-session-123".to_string();
    let secret = sm.register_session(sid.clone(), "index.nhtml".to_string()).await.expect("Register failed");
    assert_eq!(secret.len(), 32);
    
    // 2. Mise à jour d'un nœud
    let ver = sm.update_node(sid.clone(), 1, "counter".to_string(), "10".to_string(), false).await.expect("Update failed");
    assert_eq!(ver, 1);
    
    // 3. Vérification des nœuds
    let nodes = sm.get_all_nodes(sid.clone()).await.expect("Get nodes failed");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].2, "counter");
    assert_eq!(nodes[0].3, "10");
    
    // 4. Test de persistance réelle
    drop(sm);
    let sm2 = SessionManager::new(db_uri).await.expect("Failed to re-init SM");
    let nodes2 = sm2.get_all_nodes(sid.clone()).await.expect("Get nodes failed");
    assert_eq!(nodes2.len(), 1);
    assert_eq!(nodes2[0].3, "10");
    
    drop(sm2);
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_v0_7_1_sqlx_rooms() {
    let db_uri = "sqlite:test_rooms_v071.db";
    let db_path = std::path::Path::new("test_rooms_v071.db");
    let _ = std::fs::remove_file(db_path);

    let sm = SessionManager::new(db_uri).await.expect("Failed to init SM");
    
    let sid = "user1".to_string();
    sm.join_room(sid.clone(), "room1".to_string()).await.expect("Join failed");
    
    let rooms = sm.get_session_rooms(sid.clone()).await.expect("Get rooms failed");
    assert!(rooms.contains(&"room1".to_string()));
    
    sm.leave_room(sid.clone(), "room1".to_string()).await.expect("Leave failed");
    
    drop(sm);
    let _ = std::fs::remove_file(db_path);
}
