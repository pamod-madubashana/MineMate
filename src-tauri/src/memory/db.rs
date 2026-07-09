use rusqlite::{Connection, params};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub first_seen: String,
    pub last_seen: String,
    pub preferences: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub dimension: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub player: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub id: i64,
    pub name: String,
    pub data: String,
    pub author: Option<String>,
    pub created: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
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

    // Player operations
    pub fn save_player(&self, name: &str) -> Result<Player, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO players (name, first_seen, last_seen) VALUES (?1, COALESCE((SELECT first_seen FROM players WHERE name = ?1), ?2), ?3)",
            params![name, now, now],
        )?;

        let player = conn.query_row(
            "SELECT id, name, first_seen, last_seen, preferences FROM players WHERE name = ?1",
            params![name],
            |row| {
                Ok(Player {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    first_seen: row.get(2)?,
                    last_seen: row.get(3)?,
                    preferences: row.get(4)?,
                })
            },
        )?;

        Ok(player)
    }

    pub fn get_player(&self, name: &str) -> Result<Option<Player>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, name, first_seen, last_seen, preferences FROM players WHERE name = ?1",
            params![name],
            |row| {
                Ok(Player {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    first_seen: row.get(2)?,
                    last_seen: row.get(3)?,
                    preferences: row.get(4)?,
                })
            },
        );

        match result {
            Ok(player) => Ok(Some(player)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_players(&self) -> Result<Vec<Player>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, first_seen, last_seen, preferences FROM players ORDER BY last_seen DESC")?;
        let players = stmt.query_map([], |row| {
            Ok(Player {
                id: row.get(0)?,
                name: row.get(1)?,
                first_seen: row.get(2)?,
                last_seen: row.get(3)?,
                preferences: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(players)
    }

    // Location operations
    pub fn save_location(&self, name: &str, x: i32, y: i32, z: i32, dimension: &str, description: &str) -> Result<Location, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO locations (name, x, y, z, dimension, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, x, y, z, dimension, description],
        )?;

        let location = conn.query_row(
            "SELECT id, name, x, y, z, dimension, description FROM locations WHERE name = ?1 AND x = ?2 AND y = ?3 AND z = ?4",
            params![name, x, y, z],
            |row| {
                Ok(Location {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    x: row.get(2)?,
                    y: row.get(3)?,
                    z: row.get(4)?,
                    dimension: row.get(5)?,
                    description: row.get(6)?,
                })
            },
        )?;

        Ok(location)
    }

    pub fn list_locations(&self) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, x, y, z, dimension, description FROM locations")?;
        let locations = stmt.query_map([], |row| {
            Ok(Location {
                id: row.get(0)?,
                name: row.get(1)?,
                x: row.get(2)?,
                y: row.get(3)?,
                z: row.get(4)?,
                dimension: row.get(5)?,
                description: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(locations)
    }

    // History operations
    pub fn log_event(&self, event_type: &str, player: Option<&str>, details: &str) -> Result<HistoryEntry, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO history (timestamp, event_type, player, details) VALUES (?1, ?2, ?3, ?4)",
            params![now, event_type, player, details],
        )?;

        let entry = conn.query_row(
            "SELECT id, timestamp, event_type, player, details FROM history ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type: row.get(2)?,
                    player: row.get(3)?,
                    details: row.get(4)?,
                })
            },
        )?;

        Ok(entry)
    }

    pub fn get_history(&self, limit: u32) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, timestamp, event_type, player, details FROM history ORDER BY timestamp DESC LIMIT ?1")?;
        let history = stmt.query_map(params![limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                player: row.get(3)?,
                details: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(history)
    }

    // Blueprint operations
    pub fn save_blueprint(&self, name: &str, data: &str, author: &str) -> Result<Blueprint, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO blueprints (name, data, author, created) VALUES (?1, ?2, ?3, ?4)",
            params![name, data, author, now],
        )?;

        let blueprint = conn.query_row(
            "SELECT id, name, data, author, created FROM blueprints WHERE name = ?1",
            params![name],
            |row| {
                Ok(Blueprint {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    data: row.get(2)?,
                    author: row.get(3)?,
                    created: row.get(4)?,
                })
            },
        )?;

        Ok(blueprint)
    }

    pub fn list_blueprints(&self) -> Result<Vec<Blueprint>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, data, author, created FROM blueprints")?;
        let blueprints = stmt.query_map([], |row| {
            Ok(Blueprint {
                id: row.get(0)?,
                name: row.get(1)?,
                data: row.get(2)?,
                author: row.get(3)?,
                created: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(blueprints)
    }

    pub fn get_blueprint(&self, name: &str) -> Result<Option<Blueprint>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, name, data, author, created FROM blueprints WHERE name = ?1",
            params![name],
            |row| {
                Ok(Blueprint {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    data: row.get(2)?,
                    author: row.get(3)?,
                    created: row.get(4)?,
                })
            },
        );

        match result {
            Ok(blueprint) => Ok(Some(blueprint)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
