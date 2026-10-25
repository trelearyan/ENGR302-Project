// Loads data/woolworths_products.json into data/shopwise.db.
// Only touches Woolworths rows, so Pak'nSave and New World stay as they are.
//
// Run from the repo root:
//   cargo run -p database --example import_woolworths

use rusqlite::{params, Connection};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Product {
    name: String,
    price: f64,
    member_price: Option<f64>,
    volume_size: Option<String>,
    image_url: Option<String>,
}

// The scrape left site badges glued onto some names, e.g.
// "Bought beforeWoolworths Fresh Broccoli HeadBought before". Strip them here.
fn clean_name(raw: &str) -> String {
    const BADGES: [&str; 6] = [
        "Bought before",
        "Promoted",
        "LOW PRICE",
        "Member price",
        "Prices dropped",
        "Everyday Low Price",
    ];
    let mut s = raw.trim().to_string();
    loop {
        let mut changed = false;
        for b in BADGES {
            let pre = s.strip_prefix(b).map(|x| x.trim().to_string());
            if let Some(r) = pre {
                s = r;
                changed = true;
                break;
            }
            let suf = s.strip_suffix(b).map(|x| x.trim().to_string());
            if let Some(r) = suf {
                s = r;
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_unit(w: &str) -> bool {
    matches!(w, "kg" | "g" | "ml" | "l" | "pk" | "pack" | "ea")
}

// Pulls a size like "500g", "2 x 95g", "1.5l" or "20 Pack" out of a cleaned name.
fn parse_size(name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    for i in 0..words.len() {
        let w = words[i].to_lowercase();
        let digits_end = w
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(w.len());
        let (num, unit) = w.split_at(digits_end);
        if num.is_empty() || num == "." {
            continue;
        }
        if !unit.is_empty() && is_unit(unit) {
            // "2 x 95g"
            if i >= 2 && words[i - 1].eq_ignore_ascii_case("x") {
                let prev = words[i - 2];
                if !prev.is_empty() && prev.chars().all(|c| c.is_ascii_digit()) {
                    return Some(format!("{} x {}", prev, words[i]));
                }
            }
            return Some(words[i].to_string());
        }
        if unit.is_empty() && i + 1 < words.len() && num.chars().all(|c| c.is_ascii_digit()) {
            let next = words[i + 1].to_lowercase();
            if matches!(next.as_str(), "pack" | "pk" | "ea") {
                return Some(format!("{} {}", words[i], words[i + 1]));
            }
        }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("data");
    let json_path = data.join("woolworths_products.json");
    let db_path = data.join("shopwise.db");

    let products: Vec<Product> = serde_json::from_str(&std::fs::read_to_string(&json_path)?)?;
    if products.is_empty() {
        return Err("woolworths_products.json is empty, not touching the database".into());
    }
    println!("Read {} products from {}", products.len(), json_path.display());

    let mut conn = Connection::open(&db_path)?;

    // make sure the member_price column exists (add_loyalty_schema adds it too)
    let has_col: bool = {
        let mut stmt = conn.prepare("PRAGMA table_info(products)")?;
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<Result<_, _>>()?;
        names.iter().any(|n| n == "member_price")
    };
    if !has_col {
        conn.execute("ALTER TABLE products ADD COLUMN member_price REAL", [])?;
    }

    let supermarket_id: i64 = conn.query_row(
        "SELECT id FROM supermarkets WHERE chain LIKE '%oolworths%' LIMIT 1",
        [],
        |r| r.get(0),
    )?;

    let tx = conn.transaction()?;
    let deleted = tx.execute("DELETE FROM products WHERE supermarket_id = ?1", params![supermarket_id])?;
    println!("Removed {deleted} old Woolworths rows");

    let mut inserted = 0usize;
    let mut with_member = 0usize;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO products (supermarket_id, name, price, volume_size, image_url, member_price)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for p in &products {
            let name = clean_name(&p.name);
            if name.is_empty() {
                continue;
            }
            let size = parse_size(&name).or_else(|| p.volume_size.clone());
            if inserted < 5 {
                println!("  sample: {} | {:?} | ${:.2} | member {:?}", name, size, p.price, p.member_price);
            }
            stmt.execute(params![
                supermarket_id,
                name,
                p.price,
                size,
                p.image_url,
                p.member_price
            ])?;
            inserted += 1;
            if p.member_price.is_some() {
                with_member += 1;
            }
        }
    }

    // Woolworths' loyalty programme is called Everyday Rewards
    let renamed = tx.execute(
        "UPDATE loyalty_programmes SET name = 'Everyday Rewards'
         WHERE supermarket_id = ?1 AND name = 'Woolworths Rewards'",
        params![supermarket_id],
    )?;
    if renamed > 0 {
        println!("Renamed loyalty programme to Everyday Rewards");
    }

    tx.commit()?;
    println!("Inserted {inserted} Woolworths products.");
    println!("Of those, {with_member} have an Everyday Rewards price.");
    Ok(())
}