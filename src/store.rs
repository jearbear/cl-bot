use rusqlite::Connection;

use std::sync::Mutex;

use types::Result;

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn new(path: &str) -> Result<Store> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS listings (id TEXT PRIMARY KEY);",
            &[],
        )?;
        Ok(Store { conn: Mutex::new(conn) })
    }

    pub fn save(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO listings (id) VALUES (?)", &[&key])?;
        Ok(())
    }
}
