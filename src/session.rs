use tokio_rusqlite::Connection;


pub struct SessionManager {
    conn: Connection,
}

impl SessionManager {
    pub async fn new() -> tokio_rusqlite::Result<Self> {
        let db_path = "nhtml_sessions.db".to_string();

        let conn = Connection::open(db_path).await?;
        
        // Initialisation des tables
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    app_path TEXT
                )",
                [],
            )?;

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
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS patch_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT,
                    timestamp TEXT,
                    node_id INTEGER,
                    value TEXT,
                    version INTEGER
                )",
                [],
            )?;
            Ok(())
        }).await?;

        Ok(Self { conn })
    }


    pub async fn register_session(&self, session_id: String, app_path: String) -> tokio_rusqlite::Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO sessions (session_id, app_path) VALUES (?1, ?2)",
                rusqlite::params![session_id, app_path],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn get_session_path(&self, session_id: String) -> tokio_rusqlite::Result<String> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT app_path FROM sessions WHERE session_id = ?1")?;
            let res = stmt.query_row(rusqlite::params![session_id], |row| row.get(0)).unwrap_or("counter/app.php".to_string());
            Ok(res)
        }).await
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

    pub async fn update_node(&self, session_id: String, node_id: u32, tag: String, value: String) -> tokio_rusqlite::Result<u32> {
        self.conn.call(move |conn| {
            // 1. Récupérer la version actuelle
            let mut stmt = conn.prepare("SELECT version FROM nodes WHERE session_id = ?1 AND node_id = ?2")?;
            let current_version: u32 = stmt.query_row(rusqlite::params![session_id, node_id], |row| row.get(0)).unwrap_or(0);
            let new_version = current_version + 1;

            // 2. Mettre à jour (ou insérer) avec la nouvelle version, la valeur et le tag
            conn.execute(
                "INSERT OR REPLACE INTO nodes (session_id, node_id, tag, value, version) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![session_id.clone(), node_id, tag, value.clone(), new_version],
            )?;
            
            // 3. Archiver l'état dans l'historique
            conn.execute(
                "INSERT INTO patch_history (session_id, timestamp, node_id, value, version) 
                 VALUES (?1, datetime('now'), ?2, ?3, ?4)",
                rusqlite::params![session_id.clone(), node_id, value, new_version],
            )?;
            
            Ok(new_version)
        }).await
    }

    pub async fn get_node_id_by_tag(&self, session_id: String, tag: String) -> tokio_rusqlite::Result<u16> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT node_id FROM nodes WHERE session_id = ?1 AND tag = ?2")?;
            let res = stmt.query_row(rusqlite::params![session_id, tag], |row| row.get::<_, i32>(0));
            
            match res {
                Ok(id) => Ok(id as u16),
                Err(_) => {
                    // Si pas trouvé, on cherche le prochain ID libre pour cette session
                    let mut stmt = conn.prepare("SELECT IFNULL(MAX(node_id), 0) + 1 FROM nodes WHERE session_id = ?1")?;
                    let next_id: i32 = stmt.query_row(rusqlite::params![session_id], |row| row.get(0))?;
                    Ok(next_id as u16)
                }
            }
        }).await
    }

    pub async fn get_tag_by_node_id(&self, session_id: String, node_id: u32) -> tokio_rusqlite::Result<String> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT tag FROM nodes WHERE session_id = ?1 AND node_id = ?2")?;
            let tag: String = stmt.query_row(rusqlite::params![session_id, node_id], |row| row.get(0)).unwrap_or_else(|_| "".to_string());
            Ok(tag)
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

    pub async fn get_all_nodes(&self, session_id: String) -> tokio_rusqlite::Result<Vec<(u16, u32, String, String)>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT node_id, version, tag, value FROM nodes WHERE session_id = ?")?;
            let rows = stmt.query_map([session_id], |row| {
                Ok((
                    row.get::<_, i32>(0)? as u16,
                    row.get::<_, i32>(1)? as u32,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?
                ))
            })?;

            let mut results = Vec::new();
            for r in rows {
                results.push(r?);
            }
            Ok(results)
        }).await
    }

    pub async fn calculate_checksum(&self, session_id: String) -> tokio_rusqlite::Result<u32> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT node_id, value FROM nodes WHERE session_id = ?")?;
            let mut rows = stmt.query([session_id])?;
            let mut hash: u32 = 0;
            while let Some(row) = rows.next()? {
                let id: i32 = row.get(0)?;
                let val: String = row.get(1)?;
                hash = hash.wrapping_add(id as u32).wrapping_add(val.len() as u32);
            }
            Ok(hash)
        }).await
    }
}
