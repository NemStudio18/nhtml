use tokio_rusqlite::Connection;
use crate::core::{Node, SessionState, NodeType};

pub struct SessionManager {
    conn: Connection,
}

impl SessionManager {
    pub async fn new() -> tokio_rusqlite::Result<Self> {
        let paths = ["nhtml_sessions.db", "gateway/nhtml_sessions.db", "../nhtml_sessions.db", "../../nhtml_sessions.db"];
        let mut db_path = "nhtml_sessions.db".to_string();
        
        for p in paths {
            if std::path::Path::new(p).exists() {
                db_path = p.to_string();
                break;
            }
        }

        let conn = Connection::open(db_path).await?;
        
        // Initialisation des tables
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS nodes (
                    session_id TEXT,
                    node_id INTEGER,
                    tag TEXT,
                    value TEXT,
                    version INTEGER,
                    PRIMARY KEY(session_id, node_id)
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS event_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT,
                    timestamp TEXT,
                    event_type TEXT,
                    node_id INTEGER,
                    payload TEXT
                )",
                [],
            )?;
            Ok(())
        }).await?;

        Ok(Self { conn })
    }

    pub async fn log_event(&self, session_id: String, node_id: u32, event_type: String, payload: String) -> tokio_rusqlite::Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO event_log (session_id, timestamp, event_type, node_id, payload) VALUES (?1, datetime('now'), ?2, ?3, ?4)",
                rusqlite::params![session_id, event_type, node_id, payload],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn update_node(&self, session_id: String, node_id: u32, value: String) -> tokio_rusqlite::Result<u32> {
        self.conn.call(move |conn| {
            // 1. Récupérer la version actuelle
            let mut stmt = conn.prepare("SELECT version FROM nodes WHERE session_id = ?1 AND node_id = ?2")?;
            let current_version: u32 = stmt.query_row(rusqlite::params![session_id, node_id], |row| row.get(0)).unwrap_or(0);
            let new_version = current_version + 1;

            // 2. Mettre à jour (ou insérer) avec la nouvelle version et la nouvelle valeur
            conn.execute(
                "INSERT OR REPLACE INTO nodes (session_id, node_id, tag, value, version) 
                 VALUES (?1, ?2, IFNULL((SELECT tag FROM nodes WHERE session_id=?1 AND node_id=?2), ''), ?3, ?4)",
                rusqlite::params![session_id, node_id, value, new_version],
            )?;
            Ok(new_version)
        }).await
    }

    pub async fn get_node_state(&self, session_id: String, node_id: u32) -> tokio_rusqlite::Result<(String, u32)> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT value, version FROM nodes WHERE session_id = ?1 AND node_id = ?2")?;
            let res = stmt.query_row(rusqlite::params![session_id, node_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            }).unwrap_or(("".to_string(), 0));
            Ok(res)
        }).await
    }

    pub async fn get_all_nodes(&self, session_id: String) -> tokio_rusqlite::Result<Vec<(u16, u32, String)>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT node_id, version, value FROM nodes WHERE session_id = ?")?;
            let rows = stmt.query_map([session_id], |row| {
                Ok((
                    row.get::<_, i32>(0)? as u16,
                    row.get::<_, i32>(1)? as u32,
                    row.get::<_, String>(2)?
                ))
            })?;

            let mut results = Vec::new();
            for r in rows {
                results.push(r?);
            }
            Ok(results)
        }).await
    }
}
