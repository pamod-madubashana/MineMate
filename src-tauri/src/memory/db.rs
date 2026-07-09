use rusqlite::{Connection, params};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                preferences TEXT
            );

            CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                z INTEGER NOT NULL,
                dimension TEXT NOT NULL DEFAULT 'overworld',
                description TEXT
            );

            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                player TEXT,
                details TEXT
            );

            CREATE TABLE IF NOT EXISTS blueprints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                data TEXT NOT NULL,
                author TEXT,
                created TEXT NOT NULL
            );"
        )?;

        Ok(())
    }

    pub fn save_player(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO players (name, first_seen, last_seen) VALUES (?1, ?2, ?3)",
            params![name, now, now],
        )?;
        Ok(())
    }

    pub fn list_players(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM players ORDER BY last_seen DESC")?;
        let players = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(players)
    }

    pub fn save_location(&self, name: &str, x: i32, y: i32, z: i32, dimension: &str, description: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO locations (name, x, y, z, dimension, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, x, y, z, dimension, description],
        )?;
        Ok(())
    }

    pub fn list_locations(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM locations")?;
        let locations = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(locations)
    }

    pub fn log_event(&self, event_type: &str, player: Option<&str>, details: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO history (timestamp, event_type, player, details) VALUES (?1, ?2, ?3, ?4)",
            params![now, event_type, player, details],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: u32) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT event_type FROM history ORDER BY timestamp DESC LIMIT ?1")?;
        let history = stmt.query_map(params![limit], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(history)
    }

    pub fn save_blueprint(&self, name: &str, data: &str, author: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO blueprints (name, data, author, created) VALUES (?1, ?2, ?3, ?4)",
            params![name, data, author, now],
        )?;
        Ok(())
    }

    pub fn list_blueprints(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM blueprints")?;
        let blueprints = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(blueprints)
    }
}
