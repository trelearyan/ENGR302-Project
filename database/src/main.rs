//! FR-04 demo binary.
//!
//! This mocks what the Price Calculator (Samuel's component) will do
//! once it's built. The point is to prove the Rust → SQLite path
//! works end-to-end against `shopwise.db`, with no Python involved
//! at runtime.
//!
//! Run from the crate root with:
//!     cargo run --bin fr04_demo

use database::Database;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Path to the database. In the real system this will be a config
    // value; for the demo we hard-code it relative to the crate root.
    let db_path = "../data/shopwise.db";

    let db = match Database::open(db_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("could not open {db_path}: {e}");
            eprintln!("hint: run this from inside the `database/` crate folder");
            return ExitCode::FAILURE;
        }
    };

    println!();
    println!("======================================================================");
    println!("  ShopWise FR-04 demo — Rust crate reading SQLite via rusqlite");
    println!("======================================================================");
    println!();

    // ── Step 1: confirm the database is wired up ──────────────────
    println!("[1] Supermarkets loaded from shopwise.db:");
    println!("    {:<14}{:>10}", "Chain", "Products");
    println!("    {:-<14}{:->10}", "", "");
    match db.supermarket_summary() {
        Ok(rows) => {
            let total: i64 = rows.iter().map(|(_, n)| n).sum();
            for (chain, n) in &rows {
                println!("    {chain:<14}{n:>10}");
            }
            println!("    {:-<14}{:->10}", "", "");
            println!("    {:<14}{total:>10}", "Total");
        }
        Err(e) => {
            eprintln!("error querying supermarkets: {e}");
            return ExitCode::FAILURE;
        }
    }
    println!();

    // ── Step 2: a single-item lookup, the way Price Calculator
    //          will call it ───────────────────────────────────────
    let item       = "weet-bix";
    let in_range   = ["Pak'nSave", "Woolworths", "New World"];
    println!("[2] Looking up '{item}' across {} chains in range:", in_range.len());
    println!();
    match db.find_products(item, &in_range) {
        Ok(results) => {
            if results.is_empty() {
                println!("    (no matches — FR-12 'item unavailable' would fire)");
            } else {
                for r in results.iter().take(6) {
                    let vol = r.volume_size.as_deref().unwrap_or("");
                    println!(
                        "    {:<12} ${:>6.2}  {} ({})",
                        r.chain,
                        r.price,
                        truncate(&r.name, 45),
                        vol
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("query failed: {e}");
            return ExitCode::FAILURE;
        }
    }
    println!();

    // ── Step 3: shopping-list-style query, one cheapest match per
    //          chain — this is the shape of data the Price Calculator
    //          feeds into FR-06 (Cheapest), FR-07 (Fastest), FR-08
    //          (Best Value) ─────────────────────────────────────────
    let shopping_list = ["milk", "bread", "cheese", "eggs", "peanut butter"];
    println!("[3] Whole shopping list, cheapest match per chain:");
    println!();
    println!(
        "    {:<16}{:>12}{:>12}{:>12}",
        "Item", "Pak'nSave", "Woolworths", "New World"
    );
    println!("    {:-<16}{:->12}{:->12}{:->12}", "", "", "", "");

    let chains = ["Pak'nSave", "Woolworths", "New World"];
    let mut totals = [0.0_f64; 3];

    for item in &shopping_list {
        print!("    {:<16}", item);
        for (i, chain) in chains.iter().enumerate() {
            match db.cheapest_at(item, chain) {
                Ok(Some(p)) => {
                    print!("{:>12}", format!("${:.2}", p.price));
                    totals[i] += p.price;
                }
                Ok(None) => print!("{:>12}", "—"),
                Err(e) => {
                    eprintln!("\nquery failed: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        println!();
    }

    println!("    {:-<16}{:->12}{:->12}{:->12}", "", "", "", "");
    print!("    {:<16}", "TOTAL");
    for t in totals {
        print!("{:>12}", format!("${:.2}", t));
    }
    println!();
    println!();

    // ── Step 4: prove FR-12 'item unavailable' signal works ───────
    println!("[4] FR-12 signal — searching for an item that doesn't exist:");
    match db.cheapest_at("xyzzy_not_a_real_product", "Pak'nSave") {
        Ok(None) => println!("    → returned None, as expected. Price Calculator can\n      forward this as an 'item unavailable' notification."),
        Ok(Some(_)) => println!("    → unexpectedly matched something"),
        Err(e) => eprintln!("    error: {e}"),
    }
    println!();

    println!("======================================================================");
    println!("  Rust crate ✓   SQLite ✓   FR-04 query path verified end-to-end");
    println!("======================================================================");
    println!();

    ExitCode::SUCCESS
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let cut: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
