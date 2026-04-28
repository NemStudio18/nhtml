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
                    is_append INTEGER DEFAULT 0,
                    PRIMARY KEY(session_id, node_id)
                )",
                [],
            )?;

            // Table de sécurité pour HMAC et Sequence IDs
            conn.execute(
                "CREATE TABLE IF NOT EXISTS session_security (
                    session_id TEXT PRIMARY KEY,
                    secret BLOB,
                    last_seq INTEGER DEFAULT 0
                )",
                [],
            )?;

            // Migration simple : on tente d'ajouter la colonne, on ignore si elle existe déjà
            let _ = conn.execute("ALTER TABLE nodes ADD COLUMN is_append INTEGER DEFAULT 0", []);

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

            // Table des salons (Rooms)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS session_rooms (
                    session_id TEXT,
                    room_id TEXT,
                    PRIMARY KEY(session_id, room_id)
                )",
                [],
            )?;
            Ok(())
        }).await?;

        Ok(Self { conn })
    }


    pub async fn register_session(&self, session_id: String, app_path: String) -> tokio_rusqlite::Result<Vec<u8>> {
        let _sid_clone = session_id.clone();
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO sessions (session_id, app_path) VALUES (?1, ?2)",
                rusqlite::params![session_id, app_path],
            )?;

            // Générer un secret si inexistant
            let secret: Vec<u8> = conn.query_row(
                "SELECT secret FROM session_security WHERE session_id = ?1",
                [session_id.clone()],
                |row| row.get(0),
            ).unwrap_or_else(|_| {
                let mut s = vec![0u8; 32];
                getrandom::getrandom(&mut s).ok();
                conn.execute(
                    "INSERT INTO session_security (session_id, secret, last_seq) VALUES (?1, ?2, 0)",
                    rusqlite::params![session_id, s],
                ).ok();
                s
            });

            Ok(secret)
        }).await
    }

    pub async fn get_session_security(&self, session_id: String) -> tokio_rusqlite::Result<Option<(Vec<u8>, u32)>> {
        self.conn.call(move |conn| {
            let res = conn.query_row(
                "SELECT secret, last_seq FROM session_security WHERE session_id = ?1",
                [session_id],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, u32>(1)?))
            );
            match res {
                Ok(v) => Ok(Some(v)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
    }

    pub async fn update_seq_id(&self, session_id: String, seq: u32) -> tokio_rusqlite::Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "UPDATE session_security SET last_seq = ?1 WHERE session_id = ?2",
                rusqlite::params![seq, session_id],
            )?;
            Ok(())
        }).await
    }

    pub async fn update_node(&self, session_id: String, node_id: u32, tag: String, value: String, append: bool) -> tokio_rusqlite::Result<u32> {
        self.conn.call(move |conn| {
            // 1. Récupérer la version, la valeur et le flag append actuels
            let res = conn.query_row(
                "SELECT version, value, is_append FROM nodes WHERE session_id = ?1 AND node_id = ?2",
                rusqlite::params![session_id, node_id],
                |row| Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?, row.get::<_, i32>(2)?))
            );
            
            let (current_version, current_value, current_append) = res.unwrap_or((0, "".to_string(), 0));
            let new_version = current_version + 1;
            let final_value = if append { current_value + &value } else { value };
            let final_append = if append { 1 } else { current_append };

            // 2. Mettre à jour (ou insérer)
            conn.execute(
                "INSERT OR REPLACE INTO nodes (session_id, node_id, tag, value, version, is_append) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![session_id.clone(), node_id, tag, final_value.clone(), new_version, final_append],
            )?;
            
            // 3. Historique
            conn.execute(
                "INSERT INTO patch_history (session_id, timestamp, node_id, value, version) 
                 VALUES (?1, datetime('now'), ?2, ?3, ?4)",
                rusqlite::params![session_id.clone(), node_id, final_value, new_version],
            )?;
            
            Ok(new_version)
        }).await
    }

    pub async fn get_all_nodes(&self, session_id: String) -> tokio_rusqlite::Result<Vec<(u16, u32, String, String, bool)>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT node_id, version, tag, value, is_append FROM nodes WHERE session_id = ?1")?;
            let rows = stmt.query_map([session_id], |row| {
                Ok((
                    row.get::<_, i32>(0)? as u16,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)? == 1
                ))
            })?;
            
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            Ok(res)
        }).await
    }

    // --- Gestion des Salons (Rooms) ---

    pub async fn join_room(&self, session_id: String, room_id: String) -> tokio_rusqlite::Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO session_rooms (session_id, room_id) VALUES (?1, ?2)",
                rusqlite::params![session_id, room_id],
            )?;
            Ok(())
        }).await
    }

    pub async fn leave_room(&self, session_id: String, room_id: String) -> tokio_rusqlite::Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "DELETE FROM session_rooms WHERE session_id = ?1 AND room_id = ?2",
                rusqlite::params![session_id, room_id],
            )?;
            Ok(())
        }).await
    }

    pub async fn get_session_rooms(&self, session_id: String) -> tokio_rusqlite::Result<Vec<String>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT room_id FROM session_rooms WHERE session_id = ?1")?;
            let rows = stmt.query_map([session_id], |row| row.get::<_, String>(0))?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            Ok(res)
        }).await
    }

    pub async fn get_room_sessions(&self, room_id: String) -> tokio_rusqlite::Result<Vec<String>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT session_id FROM session_rooms WHERE room_id = ?1")?;
            let rows = stmt.query_map([room_id], |row| row.get::<_, String>(0))?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            Ok(res)
        }).await
    }

}
