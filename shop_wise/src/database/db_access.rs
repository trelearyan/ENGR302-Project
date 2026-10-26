use rusqlite::{Connection, Result};

const DB: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/shopwise.db"));

pub fn open_database() -> Result<Connection> {
    let mut conn: Connection = Connection::open_in_memory()?;
    conn.deserialize_bytes("main", DB)?;
    return Ok(conn);
}