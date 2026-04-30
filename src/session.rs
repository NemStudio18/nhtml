use sqlx::{AnyPool, any::AnyPoolOptions, Row};

pub struct SessionManager {
    pool: AnyPool,
}

impl SessionManager {
    pub async fn new(uri: &str) -> Result<Self, sqlx::Error> {
        // Enregistre les drivers au cas où (sqlx 0.7+)
        sqlx::any::install_default_drivers();

        let mut final_uri = uri.to_string();
        let mut max_conns = 50;

        // Robustesse pour SQLite sur Windows/Relative paths
        if uri.starts_with("sqlite:") && !uri.starts_with("sqlite::memory:") {
            max_conns = 1; // SQLite n'aime pas le multi-connexion AnyPool sans WAL
            let mut path_part = if uri.starts_with("sqlite://") {
                &uri[9..]
            } else {
                &uri[7..]
            };
            
            // Remove query params for path resolution
            let path_only = path_part.split('?').next().unwrap_or(path_part);

            if !path_only.is_empty() && !path_only.starts_with('/') && !path_only.contains(':') {
                if let Ok(curr) = std::env::current_dir() {
                    let abs = curr.join(path_only);
                    final_uri = format!("sqlite:///{}?mode=rwc", abs.to_str().unwrap().replace("\\", "/"));
                }
            } else if !uri.contains("?mode=") {
                 final_uri = format!("{}?mode=rwc", uri);
            }
        }
        
        println!("DEBUG: DB URI = {}", final_uri);

        let pool = AnyPoolOptions::new()
            .max_connections(max_conns)
            .connect(&final_uri).await?;
        
        // Initialisation des tables (Syntaxe compatible SQLite/MySQL/PG autant que possible)
        // Note: AUTOINCREMENT vs SERIAL vs AUTO_INCREMENT est un challenge.
        // On utilise des définitions simplifiées pour v0.7.1
        
        sqlx::query("CREATE TABLE IF NOT EXISTS sessions (
            session_id VARCHAR(255) PRIMARY KEY,
            app_path TEXT,
            last_seen INTEGER DEFAULT 0
        )").execute(&pool).await?;

        // Migration in case table already exists without last_seen
        sqlx::query("ALTER TABLE sessions ADD COLUMN last_seen INTEGER DEFAULT 0")
            .execute(&pool).await.ok();

        sqlx::query("CREATE TABLE IF NOT EXISTS nodes (
            session_id VARCHAR(255),
            node_id INTEGER,
            tag VARCHAR(255),
            value TEXT,
            version INTEGER,
            is_append INTEGER DEFAULT 0,
            PRIMARY KEY(session_id, node_id)
        )").execute(&pool).await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS session_security (
            session_id VARCHAR(255) PRIMARY KEY,
            secret BLOB,
            last_seq INTEGER DEFAULT 0
        )").execute(&pool).await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS event_log (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            session_id VARCHAR(255),
            timestamp VARCHAR(50),
            event_type VARCHAR(50),
            node_id INTEGER,
            payload TEXT
        )").execute(&pool).await.ok(); // ok() car AUTO_INCREMENT peut varier

        sqlx::query("CREATE TABLE IF NOT EXISTS patch_history (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            session_id VARCHAR(255),
            timestamp VARCHAR(50),
            node_id INTEGER,
            value TEXT,
            version INTEGER
        )").execute(&pool).await.ok();

        sqlx::query("CREATE TABLE IF NOT EXISTS session_rooms (
            session_id VARCHAR(255),
            room_id VARCHAR(255),
            PRIMARY KEY(session_id, room_id)
        )").execute(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn register_session(&self, session_id: String, app_path: String) -> Result<Vec<u8>, sqlx::Error> {
        // sqlx doesn't have a direct "INSERT OR REPLACE" for all drivers.
        // We do a manual check or try-catch style.
        
        let now = chrono::Utc::now().timestamp();
        sqlx::query("INSERT INTO sessions (session_id, app_path, last_seen) VALUES (?, ?, ?) 
                     ON CONFLICT(session_id) DO UPDATE SET app_path = excluded.app_path, last_seen = excluded.last_seen")
            .bind(&session_id)
            .bind(&app_path)
            .bind(now)
            .execute(&self.pool).await.ok();

        // Récupérer ou créer le secret
        let res = sqlx::query("SELECT secret FROM session_security WHERE session_id = ?")
            .bind(&session_id)
            .fetch_optional(&self.pool).await?;

        if let Some(row) = res {
            Ok(row.get(0))
        } else {
            let mut s = vec![0u8; 32];
            getrandom::getrandom(&mut s).ok();
            sqlx::query("INSERT INTO session_security (session_id, secret, last_seq) VALUES (?, ?, 0)")
                .bind(&session_id)
                .bind(&s)
                .execute(&self.pool).await?;
            Ok(s)
        }
    }

    pub async fn get_session_security(&self, session_id: String) -> Result<Option<(Vec<u8>, u32)>, sqlx::Error> {
        let row = sqlx::query("SELECT secret, last_seq FROM session_security WHERE session_id = ?")
            .bind(session_id)
            .fetch_optional(&self.pool).await?;

        if let Some(row) = row {
            Ok(Some((row.get(0), row.get::<i32, _>(1) as u32)))
        } else {
            Ok(None)
        }
    }

    pub async fn update_seq_id(&self, session_id: String, seq: u32) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("UPDATE session_security SET last_seq = ? WHERE session_id = ? AND last_seq < ?")
            .bind(seq as i32)
            .bind(session_id)
            .bind(seq as i32)
            .execute(&self.pool).await?;
        
        Ok(res.rows_affected() > 0)
    }

    pub async fn update_node(&self, session_id: String, node_id: u32, tag: String, value: String, append: bool) -> Result<u32, sqlx::Error> {
        // 1. Get current state
        let row = sqlx::query("SELECT version, value, is_append FROM nodes WHERE session_id = ? AND node_id = ?")
            .bind(&session_id)
            .bind(node_id as i32)
            .fetch_optional(&self.pool).await?;

        let (current_version, current_value, current_append) = if let Some(r) = row {
            (r.get::<i32, _>(0) as u32, r.get::<String, _>(1), r.get::<i32, _>(2))
        } else {
            (0, "".to_string(), 0)
        };

        let new_version = current_version + 1;
        let final_value = if append { current_value + &value } else { value };
        let final_append = if append { 1 } else { current_append };

        // 2. Upsert (Agnostic attempt via ON CONFLICT)
        sqlx::query("INSERT INTO nodes (session_id, node_id, tag, value, version, is_append) 
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(session_id, node_id) DO UPDATE SET 
                     tag = excluded.tag, value = excluded.value, version = excluded.version, is_append = excluded.is_append")
            .bind(&session_id)
            .bind(node_id as i32)
            .bind(tag)
            .bind(&final_value)
            .bind(new_version as i32)
            .bind(final_append)
            .execute(&self.pool).await?;

        // 3. History
        sqlx::query("INSERT INTO patch_history (session_id, timestamp, node_id, value, version) 
                     VALUES (?, datetime('now'), ?, ?, ?)")
            .bind(&session_id)
            .bind(node_id as i32)
            .bind(final_value)
            .bind(new_version as i32)
            .execute(&self.pool).await.ok();

        Ok(new_version)
    }

    pub async fn get_all_nodes(&self, session_id: String) -> Result<Vec<(u16, u32, String, String, bool)>, sqlx::Error> {
        let rows = sqlx::query("SELECT node_id, version, tag, value, is_append FROM nodes WHERE session_id = ?")
            .bind(session_id)
            .fetch_all(&self.pool).await?;

        let mut res = Vec::new();
        for row in rows {
            res.push((
                row.get::<i32, _>(0) as u16,
                row.get::<i32, _>(1) as u32,
                row.get::<String, _>(2),
                row.get::<String, _>(3),
                row.get::<i32, _>(4) == 1
            ));
        }
        Ok(res)
    }

    // --- Rooms ---
    pub async fn join_room(&self, session_id: String, room_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO session_rooms (session_id, room_id) VALUES (?, ?) ON CONFLICT DO NOTHING")
            .bind(session_id)
            .bind(room_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn leave_room(&self, session_id: String, room_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM session_rooms WHERE session_id = ? AND room_id = ?")
            .bind(session_id)
            .bind(room_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_session_rooms(&self, session_id: String) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT room_id FROM session_rooms WHERE session_id = ?")
            .bind(session_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.get(0)).collect())
    }

    pub async fn get_room_sessions(&self, room_id: String) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT session_id FROM session_rooms WHERE room_id = ?")
            .bind(room_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.get(0)).collect())
    }

    pub async fn cleanup_expired_sessions(&self, ttl_seconds: i64) -> Result<u64, sqlx::Error> {
        let threshold = chrono::Utc::now().timestamp() - ttl_seconds;
        
        let expired: Vec<String> = sqlx::query("SELECT session_id FROM sessions WHERE last_seen > 0 AND last_seen < ?")
            .bind(threshold)
            .fetch_all(&self.pool).await?
            .into_iter()
            .map(|r| r.get(0))
            .collect();

        if expired.is_empty() { return Ok(0); }

        let mut count = 0;
        for sid in expired {
            if let Ok(mut tx) = self.pool.begin().await {
                let _ = sqlx::query("DELETE FROM nodes WHERE session_id = ?").bind(&sid).execute(&mut *tx).await;
                let _ = sqlx::query("DELETE FROM patch_history WHERE session_id = ?").bind(&sid).execute(&mut *tx).await;
                let _ = sqlx::query("DELETE FROM session_security WHERE session_id = ?").bind(&sid).execute(&mut *tx).await;
                let _ = sqlx::query("DELETE FROM session_rooms WHERE session_id = ?").bind(&sid).execute(&mut *tx).await;
                let _ = sqlx::query("DELETE FROM event_log WHERE session_id = ?").bind(&sid).execute(&mut *tx).await;
                
                if let Ok(res) = sqlx::query("DELETE FROM sessions WHERE session_id = ?").bind(&sid).execute(&mut *tx).await {
                    if res.rows_affected() > 0 { count += 1; }
                }
                
                let _ = tx.commit().await;
            }
        }
        Ok(count)
    }
}
