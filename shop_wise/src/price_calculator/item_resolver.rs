use crate::price_calculator::shopping_list_parser::{CSV, ListParserError, parse_csv};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use util::coordinate::Coordinate;
use util::search::{SearchUnits, ShoppingItem, ShoppingItemQuery, match_sid_to_brand};
use util::store::{Store, StoreBrand};
use rusqlite::{Connection, Error, Result};

#[derive(Debug, PartialEq)]
struct SearchResult {
    item_id: u32,
    name: String,
    price: u32,
    store: StoreBrand,
    quantity: f32,
}

/// Resolve a ShoppingItemQuery into a ShoppingItem for each store
/// <br>
/// Returns:
/// <br>
/// Some(ShoppingItem) if the string could be resolved as a valid
/// ShoppingItem
/// <br>
/// None if the string could not be resolved
pub fn resolve(item_query: &ShoppingItemQuery) -> Option<HashMap<u32, ShoppingItem>> {
    // Split item_query into search terms
    let terms: Vec<&str> = item_query.name.split(' ').collect();
    let mut result: HashMap<u32, ShoppingItem> = HashMap::new();
    let mut search: HashMap<usize, HashMap<u32, Vec<SearchResult>>> = demo_db(&terms).unwrap();
    // For each store (future narrow to allowed)
    for i in 1..=3 {
        // stores not available at the moment, only brands
        let shop_id: &str = &i.to_string();
        let mut itemlist: HashMap<u32, SearchResult> = HashMap::new();
        // Get every item that matches some whole term of the query
        for j in 0..terms.len() {
            let term = *terms.get(j).unwrap();
            let found = search.get_mut(&j)?.remove(&i);
            if (found.is_none()) {
                continue;
            }
            // Only add item if it wasn't already in the map
            for item in found.unwrap() {
                if !itemlist.contains_key(&item.item_id) {
                    itemlist.insert(item.item_id, item);
                }
            }
        }
        // Score each item based on whether it fits the right department,
        // Multiple words of the query (multiplicative factor), and has
        // minimal other content. This should reward "Free Range Chicken Breast"
        // over "Brand Pasta Single Snack Chicken Curry Pasta & Sauce" and
        // "Wet Cat Food Chicken Breast and Herb"
        let mut best_key: i32 = i32::MIN;
        let mut best_score = i32::MIN;
        for key in itemlist.keys() {
            let item: &&SearchResult = &itemlist.get(key).unwrap();
            let name: &String = &item.name;
            let mut score: i32 = 1;
            // Score name based on search term matches and minimalism (might punish certain items - future)
            for j in 0..terms.len() {
                let mut tscore = 1;
                let term: &str = *terms.get(j).unwrap();
                name.split(' ').for_each(|x: &str| {
                    tscore += if (x.to_lowercase().contains(&term.to_lowercase())) {
                        10
                    } else {
                        -1
                    };
                });
                score *= tscore;
            }
            score *= 1000;
            // Score based on most popular category of high scoring items (narrow top results - need good already)
            // category not available at the moment
            // Grade on price & quantity matching
            score -= item.price as i32;
            score -= name.len() as i32;
            // Return best match per store
            if score > best_score {
                best_key = *key as i32;
                best_score = score;
            }
        }
        if (best_key < 0) {
            continue;
        }
        let best = itemlist.get(&(best_key as u32)).unwrap();
        result.insert(
            i,
            ShoppingItem {
                name: best.name.clone(),
                quantity: 1,
                unit: SearchUnits::EACH,
                price: best.price,
                store: Store {
                    brand: best.store,
                    location: Coordinate::from_lat_long_f32(i as f32, i as f32),
                },
            },
        );
    }

    Some(result)

    // Example search "Vegemite",1,"ea" -> return cheapest vegemite at each store, with single item and price
    // Complications (includes but is not limited to)
    // 1) "Vogel's" contains an apostrophe which is not a valid character in a sql search
    // and may not be present in the database name for the item -> no results even when clear name used
    // 2) "Butter" -> "Butter Flavoured Popcorn". "X Flavoured" almost never the right result when single
    // X search term used, but next to impossible to filter out all the variations of such, dilutes both
    // department specificity and price selection
    // 3) "Chicken Breast",1,"l" "l" is not a valid quantity of chicken -> how to handle it.
    // Sometimes this is more reasonable like searching for 3L of water, but some things are sold by the
    // pack and this makes conversion difficult. Don't want to select $48 pack of canned sparkling water
    // because couldn't resolve the $4 12 pack of bottled water when user search query contains a volume
    // (or weight)
    // 4) "Eggs",6,"ea" -> "Countdown eggs half dozen barn size 6","6pk". need to parse the quantity
    // information which will be different for every chain and type and compare to the users queried amount
    // (e.g. what if quanity was 1ea, cannot possible try to obtain info from name, i.e. parsing "half dozen")
    // 5) '&' vs 'and', 'Large' vs 'Family' vs 'Share', synonomous terms confusing exact match searching without
    // table or context (like LLM token co-ordinates)
    // 6) very similar product names that refer to different versions of the same product - e.g. "Vanilla Coke"
    // vs "Vanilla Coke Zero Sugar"
}

#[cfg(not(target_arch = "wasm32"))]
const DB_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/demo_db/shopwise.db");

#[cfg(target_arch = "wasm32")]
const DB: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/demo_db/shopwise.db"));

fn open_database() -> Result<Connection> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Connection::open(DB_PATH)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let mut conn = Connection::open_in_memory()?;
        conn.deserialize_bytes("main", DB)?;
        Ok(conn)
    }
}

/// Mock the sqlite database by using a python version and some jank commands
fn demo_db(search_terms: &[&str]) -> Result<HashMap<usize, HashMap<u32, Vec<SearchResult>>>> {
    let conn = open_database()?;
    // Check Database is loaded correctly

    let mut result: HashMap<usize, HashMap<u32, Vec<SearchResult>>> = HashMap::new();
    for term_i in 0..search_terms.len() {
        let search_term = search_terms.get(term_i).unwrap();
        let mut term_results: HashMap<u32, Vec<SearchResult>> = HashMap::new();
        for i in 1..=3 {
            let shop_id = &i.to_string();
            let sql_query = "
                SELECT id, supermarket_id, name, price, volume_size
                FROM products p
                WHERE p.supermarket_id = ?1
                AND LOWER(p.name) LIKE ?2
            ";
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            let mut stm = conn.prepare(sql_query)?;
            let res = stm.query_map( 
                rusqlite::params! {i, search_pattern,},
                |row: &rusqlite::Row<'_>|->Result<SearchResult, Error>{
                    Ok(SearchResult {
                        item_id: row.get(0)?,
                        name: row.get(2)?,
                        price: (row.get::<usize,f32>(3)? * 100.0) as u32,
                        store: match_sid_to_brand(row.get(1)?).unwrap(),
                        quantity: 1.0,
                    })
            })?;
            let shop_results: Vec<SearchResult> = res.map(|f: std::prelude::v1::Result<SearchResult, Error>|->SearchResult{return f.unwrap();}).collect();
            term_results.insert(i, shop_results);
        }
        result.insert(term_i, term_results);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::price_calculator::item_resolver::resolve;
    use util::search::ShoppingItemQuery;

    #[test]
    fn test_result() {
        resolve(&ShoppingItemQuery {
            name: String::from("Eggs"),
            quantity: 1,
            unit: String::from("ea"),
        });
    }

    #[test]
    fn test_weetbix() {
        {
            let res = resolve(&ShoppingItemQuery {
                name: String::from("Weetbix"),
                quantity: 1,
                unit: String::from("ea"),
            })
            .unwrap();
            assert_eq!(true, res.contains_key(&2));
            assert_eq!(false, res.contains_key(&1));
            assert_eq!(false, res.contains_key(&3));
        }
        {
            let res = resolve(&ShoppingItemQuery {
                name: String::from("Weet-Bix"),
                quantity: 1,
                unit: String::from("ea"),
            })
            .unwrap();
            assert_eq!(false, res.contains_key(&2));
            assert_eq!(true, res.contains_key(&1));
            assert_eq!(true, res.contains_key(&3));
        }
    }

    #[test]
    fn test_dip() {
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Onion Soup"),
            quantity: 1,
            unit: String::from("ea"),
        })
        .unwrap(); //Maggi Onion Soup
        assert_eq!("Maggi Onion Soup", res.get(&1).unwrap().name);
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Reduced Cream"),
            quantity: 1,
            unit: String::from("ea"),
        })
        .unwrap();
        assert_eq!("Pams Reduced Cream", res.get(&1).unwrap().name);
        assert_eq!("countdown reduced cream ", res.get(&2).unwrap().name);
    }
}
