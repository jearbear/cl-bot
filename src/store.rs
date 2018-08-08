use rusqlite::Connection;

use std::sync::Mutex;

use types::*;

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
        Ok(Store {
            conn: Mutex::new(conn),
        })
    }

    pub fn new_in_memory() -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS listings (id TEXT PRIMARY KEY);",
            &[],
        )?;
        Ok(Store {
            conn: Mutex::new(conn),
        })
    }

    pub fn exists(&self, key: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT * FROM listings WHERE id = (?)", &[&key], |_| {})
            .is_ok()
    }

    pub fn save(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO listings (id) VALUES (?)", &[&key])?;
        Ok(())
    }
}
