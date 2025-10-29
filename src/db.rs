use rusqlite::{Connection, Result, params};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub time: String,
    pub created_at: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                time TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_reminder(&self, title: String, description: String, time: String) -> Result<Reminder> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO reminders (title, description, time, created_at) VALUES (?, ?, ?, ?)",
            params![&title, &description, &time, &now],
        )?;
        
        let id = self.conn.last_insert_rowid() as i32;
        Ok(Reminder {
            id,
            title,
            description,
            time,
            created_at: now,
        })
    }

    pub fn get_all_reminders(&self) -> Result<Vec<Reminder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, time, created_at FROM reminders ORDER BY time ASC"
        )?;
        
        let reminders = stmt.query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                time: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for reminder in reminders {
            result.push(reminder?);
        }
        Ok(result)
    }

    pub fn update_reminder(&self, id: i32, title: String, description: String, time: String) -> Result<()> {
        self.conn.execute(
            "UPDATE reminders SET title = ?, description = ?, time = ? WHERE id = ?",
            params![&title, &description, &time, id],
        )?;
        Ok(())
    }

    pub fn delete_reminder(&self, id: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM reminders WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }
}
