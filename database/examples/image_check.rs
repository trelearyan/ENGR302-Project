// Shows how many products per chain have an image link, plus a few samples.
//
// Run from the repo root:
//   cargo run -p database --example image_check

use rusqlite::Connection;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data")
        .join("shopwise.db");
    let conn = Connection::open(&db_path)?;

    println!("Image links per chain:");
    let mut stmt = conn.prepare(
        "SELECT s.chain, COUNT(*), SUM(CASE WHEN p.image_url IS NOT NULL AND p.image_url != '' THEN 1 ELSE 0 END)
         FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
         GROUP BY s.chain ORDER BY s.chain",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
    })?;
    for row in rows {
        let (chain, total, with_img) = row?;
        println!("  {chain}: {with_img} of {total} have an image link");
    }

    println!();
    println!("Sample links:");
    let mut stmt = conn.prepare(
        "SELECT s.chain, p.name, p.image_url
         FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
         WHERE p.image_url IS NOT NULL AND p.image_url != ''
         GROUP BY s.chain",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;
    for row in rows {
        let (chain, name, url) = row?;
        println!("  {chain} | {name}");
        println!("    {url}");
    }
    Ok(())
}