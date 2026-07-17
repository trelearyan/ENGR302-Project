//! ShopWise database crate.
//!
//! This crate wraps the SQLite database (`shopwise.db`) and exposes
//! a small, typed API for the rest of the project to query. The Price
//! Calculator (Samuel Smith) is the primary consumer; it calls these
//! functions as part of fulfilling FR-04, FR-06, FR-07, and FR-08.
//!
//! Per design doc section 6.5, the database is a single SQLite file that
//! ships with the website. Per section 6.6, access is via `rusqlite`.
//!
//! # Example
//!
//! ```no_run
//! use database::Database;
//!
//! let db = Database::open("shopwise.db").unwrap();
//! let matches = db.find_products("milk", &["Pak'nSave", "Woolworths"]).unwrap();
//! for m in matches {
//!     println!("{} at {} — ${:.2}", m.name, m.chain, m.price);
//! }
//! ```

use rusqlite::{params_from_iter, Connection, Result};

/// One row returned from a product query — the price of a single
/// product at a single supermarket chain.
#[derive(Debug, Clone)]
pub struct ProductPrice {
    pub chain:       String,
    pub name:        String,
    pub price:       f64,
    pub volume_size: Option<String>,
}

/// Handle to an open SQLite database connection.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open the SQLite file at `path`. Read-only at runtime — we never
    /// write to the database from inside the running application; new
    /// pricing data is loaded by the offline build pipeline.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// Return one row per supermarket, with the chain name and how
    /// many product rows it has. Used by the demo binary as a sanity
    /// check.
    pub fn supermarket_summary(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.chain, COUNT(p.id)
             FROM supermarkets s
             LEFT JOIN products p ON p.supermarket_id = s.id
             GROUP BY s.id
             ORDER BY s.id",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// FR-04's core query: given an item name and a list of chains the
    /// Route Planner says are in range, return every matching product
    /// at those chains, cheapest first.
    ///
    /// This is the function the Price Calculator will call. For each
    /// shopping list item, it asks: "what's available, where, and for
    /// how much?" — then ranks scenarios from there (FR-06, FR-07, FR-08).
    pub fn find_products(
        &self,
        item_query: &str,
        in_range_chains: &[&str],
    ) -> Result<Vec<ProductPrice>> {
        if in_range_chains.is_empty() {
            return Ok(Vec::new());
        }

        // Build a parameter list of the form (?, ?, ?) sized to the
        // number of chains we were given. We avoid string-formatting
        // the values directly so the query stays parameterised against
        // SQL injection.
        let placeholders = vec!["?"; in_range_chains.len()].join(", ");
        let sql = format!(
            "SELECT s.chain, p.name, p.price, p.volume_size
             FROM products p
             JOIN supermarkets s ON s.id = p.supermarket_id
             WHERE s.chain IN ({placeholders})
               AND LOWER(p.name) LIKE LOWER(?)
             ORDER BY p.price ASC
             LIMIT 50"
        );

        // Combine the chain names and the LIKE pattern into one
        // parameter iterator. Extra WORD
        let pattern = format!("%{item_query}%");
        let mut params: Vec<&dyn rusqlite::ToSql> = in_range_chains
            .iter()
            .map(|c| c as &dyn rusqlite::ToSql)
            .collect();
        params.push(&pattern);

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params_from_iter(params.iter()), |row| {
                Ok(ProductPrice {
                    chain:       row.get(0)?,
                    name:        row.get(1)?,
                    price:       row.get(2)?,
                    volume_size: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Convenience: for a given item and chain, return the cheapest
    /// matching product. Returns `None` if no match exists, which is
    /// how FR-12 (Item Unavailable Notification) gets its signal.
    pub fn cheapest_at(&self, item_query: &str, chain: &str) -> Result<Option<ProductPrice>> {
        let pattern = format!("%{item_query}%");
        let mut stmt = self.conn.prepare(
            "SELECT s.chain, p.name, p.price, p.volume_size
             FROM products p
             JOIN supermarkets s ON s.id = p.supermarket_id
             WHERE s.chain = ?
               AND LOWER(p.name) LIKE LOWER(?)
             ORDER BY p.price ASC
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map([chain, &pattern], |row| {
            Ok(ProductPrice {
                chain:       row.get(0)?,
                name:        row.get(1)?,
                price:       row.get(2)?,
                volume_size: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Tests (NFR-14: code review and quality checks)
// Run with: cargo test
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // These tests assume shopwise.db lives one directory up from the
    // crate root — adjust the path if your layout differs.
    const TEST_DB: &str = "../data/shopwise.db";

    #[test]
    fn opens_database() {
        let db = Database::open(TEST_DB).expect("database file should be readable");
        let summary = db.supermarket_summary().expect("query should run");
        assert_eq!(summary.len(), 3, "expected three supermarket chains");
    }

    #[test]
    fn finds_products_in_specified_chains() {
        let db = Database::open(TEST_DB).unwrap();
        let results = db
            .find_products("milk", &["Pak'nSave", "Woolworths", "New World"])
            .unwrap();
        assert!(!results.is_empty(), "milk should match something in every chain");
    }

    #[test]
    fn returns_none_for_nonsense_item() {
        let db = Database::open(TEST_DB).unwrap();
        let result = db.cheapest_at("xyzzy_nonexistent_product", "Pak'nSave").unwrap();
        assert!(result.is_none());
    }
}
