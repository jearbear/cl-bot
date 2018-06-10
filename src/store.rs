use rusqlite::Connection;

use types::Result;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(path: &str) -> Result<Store> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS listings (id TEXT PRIMARY KEY);",
            &[],
        )?;
        Ok(Store { conn: conn })
    }

    pub fn save(&self, key: &str) -> Result<()> {
        self.conn
            .execute("INSERT INTO listings (id) VALUES (?)", &[&key])?;
        Ok(())
    }
}
