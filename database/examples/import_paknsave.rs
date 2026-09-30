use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;

fn main() {
    let conn = Connection::open("../data/shopwise.db")
        .expect("could not open database — run this from inside the database/ folder");

    let json_text = fs::read_to_string("../data/paknsave.json")
        .expect("could not read ../data/paknsave.json — did you copy it there?");

    let parsed: Value = serde_json::from_str(&json_text)
        .expect("paknsave.json was not valid JSON");

    let products = parsed["products"]
        .as_array()
        .expect("expected a top-level \"products\" array");

    conn.execute(
        "INSERT OR IGNORE INTO supermarkets (chain) VALUES (?1)",
        params!["Pak'nSave"],
    )
    .expect("failed to insert supermarket row");

    let supermarket_id: i64 = conn
        .query_row(
            "SELECT id FROM supermarkets WHERE chain = ?1",
            params!["Pak'nSave"],
            |row| row.get(0),
        )
        .expect("could not find Pak'nSave in supermarkets table");

    let deleted = conn
        .execute(
            "DELETE FROM products WHERE supermarket_id = ?1",
            params![supermarket_id],
        )
        .expect("failed to clear old Pak'nSave products");
    println!("Cleared {} existing Pak'nSave product rows.", deleted);

    let tx = conn.unchecked_transaction().expect("could not start transaction");
    let mut inserted = 0;
    let mut skipped = 0;

    for product in products {
        let name = match product["name"].as_str() {
            Some(n) if !n.is_empty() => n,
            _ => {
                skipped += 1;
                continue;
            }
        };
        let price = match product["price"].as_f64() {
            Some(p) => p,
            None => {
                skipped += 1;
                continue;
            }
        };
        let volume_size = product["unit"].as_str();

        tx.execute(
            "INSERT INTO products (supermarket_id, name, price, volume_size, image_url)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![supermarket_id, name, price, volume_size],
        )
        .expect("failed to insert product");

        inserted += 1;
    }

    tx.commit().expect("failed to commit transaction");

    println!("Inserted {} products ({} skipped due to missing name/price).", inserted, skipped);

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE supermarket_id = ?1",
            params![supermarket_id],
            |row| row.get(0),
        )
        .expect("count query failed");
    println!("Total Pak'nSave products now in database: {}", total);
}