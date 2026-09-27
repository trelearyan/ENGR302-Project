//! FR-04 demo binary.
//!
//! This mocks what the Price Calculator (Samuel's component) will do
//! once it's built. The point is to prove the Rust -> SQLite path
//! works end-to-end against `shopwise.db`, with no Python involved
//! at runtime.
//!
//! Run from the crate root with:
//!     cargo run --bin fr04_demo

use database::{chain_name, Database};
use std::process::ExitCode;
use util::coordinate::Coordinate;
use util::cost::Cost;
use util::store::{Store, StoreBrand};

fn main() -> ExitCode {
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
    println!("  ShopWise FR-04 demo - Rust crate reading SQLite via rusqlite");
    println!("======================================================================");
    println!();

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

    // Placeholder coordinates — the real ones will come from the Route
    // Planner once it's wired up. Only `brand` matters for this demo.
    let in_range_stores = [
        Store { brand: StoreBrand::Paknsave, location: Coordinate::WELLINGTON },
        Store { brand: StoreBrand::Woolworths, location: Coordinate::WELLINGTON },
        Store { brand: StoreBrand::Newworld, location: Coordinate::WELLINGTON },
    ];

    let item = "weet-bix";
    println!("[2] Looking up '{item}' across {} chains in range:", in_range_stores.len());
    println!();
    match db.find_products(item, &in_range_stores) {
        Ok(results) => {
            if results.is_empty() {
                println!("    (no matches - FR-12 'item unavailable' would fire)");
            } else {
                for r in results.iter().take(6) {
                    let vol = r
                        .volume_size
                        .as_ref()
                        .map(|q| q.to_string())
                        .unwrap_or_default();
                    println!(
                        "    {:<12} ${:>6}  {} ({})",
                        chain_name(r.chain),
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

    let shopping_list = ["milk", "bread", "cheese", "eggs", "peanut butter"];
    println!("[3] Whole shopping list, cheapest match per chain:");
    println!();
    println!(
        "    {:<16}{:>12}{:>12}{:>12}",
        "Item", "Pak'nSave", "Woolworths", "New World"
    );
    println!("    {:-<16}{:->12}{:->12}{:->12}", "", "", "", "");

    let chains = [StoreBrand::Paknsave, StoreBrand::Woolworths, StoreBrand::Newworld];
    let mut totals: [Cost; 3] = [Cost::default(), Cost::default(), Cost::default()];

    for item in &shopping_list {
        print!("    {:<16}", item);
        for (i, chain) in chains.iter().enumerate() {
            match db.cheapest_at(item, &[*chain]) {
                Ok(Some(p)) => {
                    print!("{:>12}", format!("${}", p.price));
                    totals[i] += p.price;
                }
                Ok(None) => print!("{:>12}", "-"),
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
    for t in &totals {
        print!("{:>12}", format!("${t}"));
    }
    println!();
    println!();

    println!("[4] FR-12 signal - searching for an item that doesn't exist:");
    match db.cheapest_at("xyzzy_not_a_real_product", &[StoreBrand::Paknsave]) {
        Ok(None) => println!("    -> returned None, as expected. Price Calculator can\n      forward this as an 'item unavailable' notification."),
        Ok(Some(_)) => println!("    -> unexpectedly matched something"),
        Err(e) => eprintln!("    error: {e}"),
    }
    println!();

    println!("======================================================================");
    println!("  Rust crate OK   SQLite OK   FR-04 query path verified end-to-end");
    println!("======================================================================");
    println!();

    ExitCode::SUCCESS
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let cut: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{cut}...")
    }
}