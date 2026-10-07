use rusqlite::{params, Connection};

fn main() {
    let conn = Connection::open("../data/shopwise.db")
        .expect("could not open database — run this from inside the database/ folder");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS loyalty_programmes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            supermarket_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(supermarket_id, name),
            FOREIGN KEY (supermarket_id) REFERENCES supermarkets(id)
        )",
        [],
    )
    .expect("failed to create loyalty_programmes table");

    // Only add the column if it isn't there yet, so this is safe to re-run.
    let has_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('products') WHERE name = 'member_price'",
            [],
            |row| row.get(0),
        )
        .expect("could not inspect products table");

    if has_column == 0 {
        conn.execute("ALTER TABLE products ADD COLUMN member_price REAL", [])
            .expect("failed to add member_price column");
        println!("Added products.member_price column.");
    } else {
        println!("products.member_price already exists, skipping.");
    }

    let programmes = [
        ("New World", "New World Clubcard"),
        ("Woolworths", "Woolworths Rewards"),
    ];

    for (chain, programme) in programmes {
        conn.execute(
            "INSERT OR IGNORE INTO supermarkets (chain) VALUES (?1)",
            params![chain],
        )
        .expect("failed to insert supermarket row");

        let supermarket_id: i64 = conn
            .query_row(
                "SELECT id FROM supermarkets WHERE chain = ?1",
                params![chain],
                |row| row.get(0),
            )
            .expect("could not find supermarket");

        conn.execute(
            "INSERT OR IGNORE INTO loyalty_programmes (supermarket_id, name) VALUES (?1, ?2)",
            params![supermarket_id, programme],
        )
        .expect("failed to insert loyalty programme");
    }

    let mut stmt = conn
        .prepare(
            "SELECT s.chain, l.name FROM loyalty_programmes l
             JOIN supermarkets s ON s.id = l.supermarket_id",
        )
        .expect("prepare failed");
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .expect("query failed");

    println!("Loyalty programmes now in database:");
    for row in rows {
        let (chain, name) = row.unwrap();
        println!("  {} -> {}", chain, name);
    }
}