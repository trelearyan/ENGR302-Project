use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;

fn main() {
    let conn = Connection::open("../data/shopwise.db")
        .expect("could not open database — run this from inside the database/ folder");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS store_locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            supermarket_id INTEGER NOT NULL,
            external_id TEXT,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            FOREIGN KEY (supermarket_id) REFERENCES supermarkets(id)
        )",
        [],
    )
    .expect("failed to create store_locations table");

    let json_text = fs::read_to_string("../data/newworld_locations.json")
        .expect("could not read ../data/newworld_locations.json — did you copy it there?");

    let parsed: Value = serde_json::from_str(&json_text)
        .expect("newworld_locations.json was not valid JSON");

    let stores = parsed["stores"]
        .as_array()
        .expect("expected a top-level \"stores\" array");

    conn.execute(
        "INSERT OR IGNORE INTO supermarkets (chain) VALUES (?1)",
        params!["New World"],
    )
    .expect("failed to insert supermarket row");

    let supermarket_id: i64 = conn
        .query_row(
            "SELECT id FROM supermarkets WHERE chain = ?1",
            params!["New World"],
            |row| row.get(0),
        )
        .expect("could not find New World in supermarkets table");

    let deleted = conn
        .execute(
            "DELETE FROM store_locations WHERE supermarket_id = ?1",
            params![supermarket_id],
        )
        .expect("failed to clear old New World locations");
    println!("Cleared {} existing New World location rows.", deleted);

    let tx = conn.unchecked_transaction().expect("could not start transaction");
    let mut inserted = 0;

    for store in stores {
        let external_id = store["id"].as_str();
        let name = store["name"].as_str().unwrap_or_default();
        let address = store["address"].as_str().unwrap_or_default();
        let latitude = store["latitude"].as_f64().unwrap_or(0.0);
        let longitude = store["longitude"].as_f64().unwrap_or(0.0);

        tx.execute(
            "INSERT INTO store_locations (supermarket_id, external_id, name, address, latitude, longitude)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![supermarket_id, external_id, name, address, latitude, longitude],
        )
        .expect("failed to insert store location");

        inserted += 1;
    }

    tx.commit().expect("failed to commit transaction");

    println!("Inserted {} New World store locations.", inserted);

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM store_locations WHERE supermarket_id = ?1",
            params![supermarket_id],
            |row| row.get(0),
        )
        .expect("count query failed");
    println!("Total New World locations now in database: {}", total);
}