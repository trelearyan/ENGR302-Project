use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::hash_map::Iter as HashIter;
use std::slice::Iter as VecIter;
use util::coordinate::Coordinate;
use util::cost::Cost;
use util::search::{SearchUnits, ShoppingItem, ShoppingItemQuery, match_sid_to_brand};
use util::store::{Store, StoreBrand};

use rusqlite::{Connection, Error, Result};
use regex::Regex;


#[derive(Debug, PartialEq)]
pub struct SantizedSearchResult {
    name: String,
    unit: SearchUnits,
    image_url: String,
}

#[derive(Debug, PartialEq)]
struct SearchResult {
    item_id: u32,
    name: String,
    price: u32,
    store: StoreBrand, // For now as database is mocked
    quantity: u32,
    unit: SearchUnits,
    image_url: String,
}


/// Return a list of the best matching items for the given search
/// so the user can pick the one that best matches what they mean
/// or further refine their search if the results aren't as desired
/// <br>
/// Returns:
/// <br>
/// Some(Vec<ShoppingItem>>) where the Vec is num_options long
/// Listed in descending *calculated* relevancy as index increases
/// <br>
/// None if the string could not be resolved
pub fn search(item_query: &ShoppingItemQuery, num_options: u32) -> Option<Vec<SantizedSearchResult>> {
    let terms: Vec<&str> = item_query.name.split(' ').collect();
    let res = demo_db(&terms).unwrap();
    // Flatten and collect returned items with their scores
    let mut list = res.iter()
        .flat_map(|search_map: (&usize, &HashMap<u32, Vec<SearchResult>>)|->
            HashIter<'_, u32, Vec<SearchResult>> {search_map.1.iter()})
        .flat_map(|store_map: (&u32, &Vec<SearchResult>)|->
            VecIter<'_, SearchResult> {store_map.1.iter()})
        .map(|search_result: &SearchResult|->
            (&SearchResult, i32) {(search_result, score_item(&terms, &search_result, search_result.price))})
        .collect::<Vec<_>>();
    // Sort descending
    list.sort_by(|a , b|->Ordering {b.1.cmp(&a.1)});
    if list.len() >= num_options as usize {
        Some(list.iter()
            .take(num_options as usize)
            .map(|f|->SantizedSearchResult {
                SantizedSearchResult {
                    name: f.0.name.clone(),
                    unit: f.0.unit.clone(),
                    image_url: f.0.image_url.clone(),
                }})
            .collect::<Vec<SantizedSearchResult>>()
        )
    } else {
        None
    }
}

/// Resolve a ShoppingItemQuery into a ShoppingItem for each store
/// <br>
/// Returns:
/// <br>
/// Some(HashMap<StoreID (u32), ShoppingItem>) if the string could be resolved as a valid
/// ShoppingItem. This contains only entries for stores where shopping items were found
/// <br>
/// None if the string could not be resolved
#[must_use]
pub fn resolve(item_query: &ShoppingItemQuery) -> Option<HashMap<u32, ShoppingItem>> {
    // Split item_query into search terms
    let terms: Vec<&str> = item_query.name.split(' ').collect();
    let mut result: HashMap<u32, ShoppingItem> = HashMap::new();
    let mut search: HashMap<usize, HashMap<u32, Vec<SearchResult>>> = demo_db(&terms).unwrap();
    let search_unit = SearchUnits::match_to_unit(&item_query.unit).unwrap();
    // For each store (future narrow to allowed)
    for i in 1..=3 {
        // stores not available at the moment, only brands
        let _shop_id: &str = &i.to_string();
        let mut itemlist: HashMap<u32, SearchResult> = HashMap::new();
        // Get every item that matches some whole term of the query
        for j in 0..terms.len() {
            let _term = *terms.get(j).unwrap();
            let found = search.get_mut(&j)?.remove(&i);
            if found.is_none()  {
                continue;
            }
            // Only add item if it wasn't already in the map
            for item in found.unwrap() {
                itemlist.entry(item.item_id).or_insert(item);
            }
        }
        // Score each item based on whether it fits the right department,
        // Multiple words of the query (multiplicative factor), and has
        // minimal other content. This should reward "Brand Free Range Chicken Breast"
        // over "Brand Pasta Single Snack Chicken Curry Pasta & Sauce" and
        // "Brand Wet Cat Food Chicken Breast and Herb"
        let mut best_key: Option<i32> = None;
        let mut best_score = i32::MIN;
        let mut best_mul: Option<u32> = None;
        for key in itemlist.keys() {
            let item: &&SearchResult = &itemlist.get(key).unwrap();
            let mul: Option<u32> = search_unit
                .scale_to_match(item_query.quantity, &item.unit, item.quantity, item.price);
            if mul.is_none() {
                continue;
            }
            let price: u32 = item.price * mul.unwrap();
            let score: i32 = score_item(&terms, item, price);
            // Return best match per store
            if score > best_score {
                best_key = Some(*key as i32);
                best_score = score;
                best_mul = mul;
            }
        }
        if best_key.is_none() {
            continue;
        }
        let best = itemlist.get(&(best_key.unwrap() as u32)).unwrap();
        result.insert(
            i,
            ShoppingItem {
                name: best.name.clone(),
                quantity: best.quantity * best_mul.unwrap(),
                unit: SearchUnits::EACH,
                price: Cost::from_cents(best.price * best_mul.unwrap()),
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
    // (e.g. what if quantity was 1ea, cannot possible try to obtain info from name, i.e. parsing "half dozen")
    // 5) '&' vs 'and', 'Large' vs 'Family' vs 'Share', synonomous terms confusing exact match searching without
    // table or context (like LLM token co-ordinates)
    // 6) very similar product names that refer to different versions of the same product - e.g. "Vanilla Coke"
    // vs "Vanilla Coke Zero Sugar"
}

fn score_item(terms: &[&str], item: &&SearchResult, price: u32) -> i32 {

    let mut score: i32 = 1;
    // Score name based on search term matches and minimalism (might punish certain items - future)
    for j in 0..terms.len() {
        let mut tscore = 1;
        let term: &str = *terms.get(j).unwrap();
        let _ = &item.name.split(' ').for_each(|x: &str| {
            tscore += if x.to_lowercase().contains(&term.to_lowercase()) {
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
    score - (price as i32) - (item.name.len() as i32)
}

const DB: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/demo_db/shopwise.db"));

fn open_database() -> Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    conn.deserialize_bytes("main", DB)?;
    Ok(conn)
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
            let _shop_id = &i.to_string();
            let sql_query = "
                SELECT id, supermarket_id, name, price, volume_size, image_url
                FROM products p
                WHERE p.supermarket_id = ?1
                AND LOWER(p.name) LIKE ?2
            ";
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            let mut stm = conn.prepare(sql_query)?;
            let res = stm.query_map( 
                rusqlite::params! {i, search_pattern,},
                |row: &rusqlite::Row<'_>|->Result<SearchResult, Error>{
                    let amt = get_unit(row.get(4).unwrap_or("ea".to_owned()));
                    Ok(SearchResult {
                        item_id: row.get(0)?,
                        name: row.get(2)?,
                        price: (row.get::<usize,f32>(3)? * 100.0) as u32,
                        store: match_sid_to_brand(row.get(1)?).unwrap(),
                        quantity: amt.0,
                        unit: amt.1,
                        image_url: row.get(5)?,
                    })
            })?;
            let shop_results: Vec<SearchResult> = res.map(|f: std::prelude::v1::Result<SearchResult, Error>|->SearchResult{f.unwrap()}).collect();
            term_results.insert(i, shop_results);
        }
        result.insert(term_i, term_results);
    }
    Ok(result)
}

const DEFAULT_UNIT: (u32, SearchUnits) = (1, SearchUnits::EACH);

/// Translate the volume sizes given by the datase into the most
/// reasonable unit, or just leave as 1 each if none can be found
/// e.g.
/// 3.5ml - (3 'ml')
/// 68g - (68, 'g')
/// 4 pk - (4, 'ea')
/// 390G - (390, 'g')
/// 36pk - (36, 'ea')
/// 170pk - (170, 'ea')
/// 720ml - (720, 'ml')
/// sugar 1.2kg - (1200, 'g')
/// 10 slices - (10, 'ea')
/// none - (1, 'ea')
fn get_unit(unit_text: String) -> (u32, SearchUnits) {
    // Lower and strip
    let sanitised: String = unit_text.trim().to_lowercase();
    // Seperate the number if it begins with one, or the two numbers if in the form 2 x 4pk
    let re = Regex::new(r"[0-9]+(?:\.[0-9]+)?").unwrap();
    let values: Vec<f32> = re.find_iter(sanitised.as_ref())
        .take(2)
        .map(|x|-> f32 {x.as_str().parse::<f32>().unwrap_or(f32::NAN)})
        .collect::<Vec<f32>>();
    if values.is_empty() || !values.get(0).unwrap().is_finite() {
        return DEFAULT_UNIT;
    }
    let mut quantity: f32 = *values.get(0).unwrap();
    let mulre = Regex::new(r"[0-9]+ x [0-9]+.*").unwrap();
    // If follows the 2 x 4pk pattern, multiply the two numbers together
    if mulre.is_match(sanitised.as_ref()) {
        let temp: f32 = *values.get(1).unwrap();
        if temp.is_finite() {
            quantity *= temp;
        }
    }
    // Make sure quantity is positive
    if quantity <= 0.0 {
        return DEFAULT_UNIT;
    }
    // Replace all the digits with whitespace, then lower and strip again
    let rep_re = Regex::new(r"[0-9]+(?:[0-9]+)?").unwrap();
    let stripped = rep_re.replace_all(sanitised.as_ref(), " ").trim().to_lowercase();
    // Search for unit names, taking care to do 'kg' before 'g' and 'ml' before 'l'
    let text_bits = stripped.split(" ").collect::<Vec<&str>>();
    // kg
    if text_bits.clone().into_iter().any(|x|->bool{x == "kg"}) {
        // If significant remainder do grams
        if quantity % 1.0 > 0.05 {
            let new_quantity = (quantity * 1000.0) as u32;
            if new_quantity >= 1 {
                return (new_quantity, SearchUnits::GRAM);
            }
            return DEFAULT_UNIT;
        }
        return (quantity as u32, SearchUnits::KILOGRAM);
    }
    // ml
    if text_bits.clone().into_iter().any(|x|->bool{x == "ml"}) {
        return (quantity as u32, SearchUnits::MILLILITRE);
    }
    // g
    if text_bits.clone().into_iter().any(|x|->bool{x == "g"}) {
        return (quantity as u32, SearchUnits::GRAM);
    }
    // l
    if text_bits.into_iter().any(|x|->bool{x == "l"}) {
        // If significant remainder do grams
        if quantity % 1.0 > 0.05 {
            let new_quantity = (quantity * 1000.0) as u32;
            if new_quantity >= 1 {
                return (new_quantity, SearchUnits::MILLILITRE);
            }
            return DEFAULT_UNIT;
        }
        return (quantity as u32, SearchUnits::LITRE);
    }
    // Otherwise return default
    DEFAULT_UNIT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result() {
        let _ = resolve(&ShoppingItemQuery {
            name: String::from("Eggs"),
            quantity: 1,
            unit: SearchUnits::EACH.to_str().to_owned(),
        });
    }

    #[test]
    fn test_weetbix() {
        {
            let res = resolve(&ShoppingItemQuery {
                name: String::from("Weetbix"),
                quantity: 1,
                unit: SearchUnits::GRAM.to_str().to_owned(),
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
                unit: SearchUnits::GRAM.to_str().to_owned(),
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
            unit: SearchUnits::GRAM.to_str().to_owned(),
        })
        .unwrap(); //Maggi Onion Soup
        assert_eq!("Maggi Onion Soup", res.get(&1).unwrap().name);
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Reduced Cream"),
            quantity: 1,
            unit: SearchUnits::MILLILITRE.to_str().to_owned(),
        })
        .unwrap();
        assert_eq!("Pams Reduced Cream", res.get(&1).unwrap().name);
        assert_eq!("countdown reduced cream ", res.get(&2).unwrap().name);
    }

    #[test]
    fn test_search() {
        assert_eq!(10, search(&ShoppingItemQuery {
            name: String::from("Milk"),
            quantity: 1,
            unit: SearchUnits::EACH.to_str().to_owned(),
        }, 10).unwrap().len());
    }

    fn simple_query(sql_query: &str) -> Vec<String> {
        let conn = open_database().unwrap();
        let mut stm = conn.prepare(sql_query).unwrap();
        stm.query_map([], |row: &rusqlite::Row<'_>| -> Result<String, Error>{
                Ok(row.get(0)?)
            }).unwrap()
            .map(|f|->String{f.unwrap_or("empty".to_owned())})
            .collect::<Vec<String>>()
    }

    #[test]
    fn test_units() {
        simple_query("SELECT volume_size FROM products").into_iter()
            .take(500)
            .for_each(|x|->(){get_unit(x);});
    }
}
