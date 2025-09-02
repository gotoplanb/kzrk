use rusqlite::{Connection, Result as SqlResult, params};
use std::collections::HashMap;
use uuid::Uuid;

use crate::systems::{GameRoom, PlayerSession};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.create_tables()?;
        Ok(db)
    }

    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> SqlResult<()> {
        // Create rooms table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create sessions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                player_id TEXT PRIMARY KEY,
                player_name TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create index on player_name for quick lookups
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_player_name ON sessions(player_name)",
            [],
        )?;

        Ok(())
    }

    pub fn save_room(&self, room: &GameRoom) -> SqlResult<()> {
        let json_data = serde_json::to_string(room)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO rooms (id, data, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![room.id.to_string(), json_data],
        )?;
        Ok(())
    }

    pub fn save_session(&self, session: &PlayerSession) -> SqlResult<()> {
        let json_data = serde_json::to_string(session)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (player_id, player_name, data, updated_at) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            params![session.player_id.to_string(), session.player_name, json_data],
        )?;
        Ok(())
    }

    pub fn load_all_rooms(&self) -> SqlResult<HashMap<Uuid, GameRoom>> {
        let mut stmt = self.conn.prepare("SELECT id, data FROM rooms")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let data: String = row.get(1)?;
            Ok((id_str, data))
        })?;

        let mut rooms = HashMap::new();
        for row in rows {
            let (id_str, data) = row?;
            if let (Ok(id), Ok(room)) = (
                Uuid::parse_str(&id_str),
                serde_json::from_str::<GameRoom>(&data),
            ) {
                rooms.insert(id, room);
            }
        }
        Ok(rooms)
    }

    pub fn load_all_sessions(&self) -> SqlResult<HashMap<Uuid, PlayerSession>> {
        let mut stmt = self.conn.prepare("SELECT player_id, data FROM sessions")?;
        let rows = stmt.query_map([], |row| {
            let player_id_str: String = row.get(0)?;
            let data: String = row.get(1)?;
            Ok((player_id_str, data))
        })?;

        let mut sessions = HashMap::new();
        for row in rows {
            let (player_id_str, data) = row?;
            if let (Ok(player_id), Ok(session)) = (
                Uuid::parse_str(&player_id_str),
                serde_json::from_str::<PlayerSession>(&data),
            ) {
                sessions.insert(player_id, session);
            }
        }
        Ok(sessions)
    }

    pub fn find_sessions_by_player_name(&self, player_name: &str) -> SqlResult<Vec<PlayerSession>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM sessions WHERE player_name = ?1")?;
        let rows = stmt.query_map([player_name], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            let data = row?;
            if let Ok(session) = serde_json::from_str::<PlayerSession>(&data) {
                sessions.push(session);
            }
        }
        Ok(sessions)
    }

    #[allow(dead_code)]
    pub fn delete_room(&self, room_id: &Uuid) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM rooms WHERE id = ?1",
            params![room_id.to_string()],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_session(&self, player_id: &Uuid) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM sessions WHERE player_id = ?1",
            params![player_id.to_string()],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup_empty_sessions(&self) -> SqlResult<usize> {
        // Remove sessions that don't have an associated room
        let count = self.conn.execute(
            "DELETE FROM sessions WHERE player_id NOT IN (
                SELECT json_extract(data, '$.players') FROM rooms
            )",
            [],
        )?;
        Ok(count)
    }
}
