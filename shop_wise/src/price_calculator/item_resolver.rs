use std::collections::HashMap;
use std::collections::hash_map::Iter as HashIter;
use std::slice::Iter as VecIter;
use std::sync::RwLock;
use lazy_static::lazy_static;
use util::cost::Cost;
use util::search::*;
use util::store::StoreBrand;

use rusqlite::{Connection, Error, Result};
use regex::Regex;

use crate::database::db_access;

// Public facing types

#[derive(Debug, PartialEq, Clone)]
pub struct SantizedSearchResult {
    pub name: String,
    pub unit: SearchUnits,
    pub quantity: u32,
    pub image_url: String,
}

// Private/Internal types

#[derive(Debug, PartialEq, Clone)]
struct SearchResult {
    item_id: u32,
    name: String,
    price: u32,
    member_price: u32,
    store: StoreBrand, // For now as database is mocked
    quantity: u32,
    unit: SearchUnits,
    image_url: String,
}
lazy_static!{
static ref QUERY_CACHE: RwLock<HashMap<String, HashMap<u32, Vec<SearchResult>>>> = RwLock::new(HashMap::new());
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
pub fn search(item_query: &str, num_options: u32) -> Option<Vec<SantizedSearchResult>> {
    let terms: Vec<&str> = item_query.split(' ').collect();
    let res = query_db(&terms).ok()?;
    // Flatten and collect returned items with their scores
    let mut list = res.iter()
        .flat_map(|search_map: (&usize, &HashMap<u32, Vec<SearchResult>>)|->
            HashIter<'_, u32, Vec<SearchResult>> {search_map.1.iter()})
        .flat_map(|store_map: (&u32, &Vec<SearchResult>)|->
            VecIter<'_, SearchResult> {store_map.1.iter()})
        .map(|search_result: &SearchResult|->
            (&SearchResult, f32) {(search_result, score_item(&terms, &search_result))})
        .collect::<Vec<_>>();
    // Sort descending
    list.sort_by(|arg0: &(&SearchResult, f32), arg1: &(&SearchResult, f32)| f32::total_cmp(&arg1.1, &arg0.1));
    if list.len() >= 1 {
        Some(list.iter()
            .take(num_options as usize)
            .map(|f|->SantizedSearchResult {
                SantizedSearchResult {
                    name: f.0.name.clone(),
                    unit: f.0.unit.clone(),
                    quantity: f.0.quantity,
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
pub fn resolve(item_query: &ShoppingItemQuery,
            edr: bool,
            club: bool,
        ) -> Option<HashMap<StoreBrand, ResolvedItem>> {
    // Split item_query into search terms
    let binding: String = item_query.name.to_lowercase().trim().to_string();
    let terms: Vec<&str> = binding.split(' ').filter(|f|->bool{f.len()>=1}).collect();
    println!("{:?}", terms);
    let mut result: HashMap<StoreBrand, ResolvedItem> = HashMap::new();
    let mut search: HashMap<usize, HashMap<u32, Vec<SearchResult>>> = query_db(&terms).unwrap();
    let search_unit = SearchUnits::match_to_unit(&item_query.unit).unwrap();
    // For each store (future narrow to allowed)
    for i in 1..=3 {
        let mut member: bool = false;
        if i == match_brand_to_sid(StoreBrand::Woolworths) {
            if edr {
                member = true;
            }
        } else {
            if club {
                member = true;
            }
        }
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
        let mut top_score = f32::MIN;
        let mut best_key: Option<i32> = None;
        let mut best_mul: Option<u32> = None;
        let mut best_price: u32 = u32::MAX;
        for key in itemlist.keys() {
            let item: &&SearchResult = &itemlist.get(key).unwrap();
            let mul: Option<u32> = search_unit
                .scale_to_match(item_query.quantity, &item.unit, item.quantity, item.price);
            if mul.is_none() {
                continue;
            }
            let price: u32 = mul.unwrap() * if member {
                item.member_price
            } else {
                item.price
            };
            let score: f32 = score_item(&terms, item);
            // Find the cheapest item within a 10% match of the best matching item
            if score > top_score * 1.1 ||
                (score > top_score * 0.91 && price < best_price) {
                best_key = Some(*key as i32);
                best_price = price;
                best_mul = mul;
            }
            if score > top_score {
                top_score = score;
            }
        }
        if best_key.is_none() {
            continue;
        }
        let best = itemlist.get(&(best_key.unwrap() as u32)).unwrap();
        result.insert(
            best.store,
            ResolvedItem {
                name: best.name.clone(),
                multiplier: best_mul.unwrap(),
                quantity: best.quantity,
                unit: best.unit.clone(),
                price: Cost::from_cents(best.price),
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

fn score_item(terms: &[&str], item: &&SearchResult) -> f32 {
    let mut score: f32 = 0.0;
    //println!("---");
    // Score name based on search term matches and minimalism (might punish certain items - future)
    // density
    let parts: Vec<&str> = item.name.split(' ').collect();
    for j in 0..parts.len() {
        let mut tscore: f32 = 1.0;
        let part: &str = *parts.get(j).unwrap();
        let _ = &terms.into_iter().for_each(|term: &&str| {
            tscore *= if part.to_lowercase().contains(&term.to_lowercase()) {
                10.0
            } else {
                0.8
            };
        });
        //print!(" {:?} = {:?}, ", part, tscore);
        score += tscore / (parts.len() as f32);
    }
    // coverage
    let mut cover = 0;
    for j in 0..terms.len() {
        let mut tscore: f32 = 1.0;
        let term: &str = *terms.get(j).unwrap();
        let _ = &item.name.split(' ').for_each(|part: &str| {
            if part.is_empty() {
                return;
            }
            tscore *= if term.to_lowercase().starts_with(&part.to_lowercase()) {
                10.0
            } else {
                0.8
            };
        });
        if tscore > 1.0 {
            cover += 1
        }
        //print!(" {:?} = {:?}, ", part, tscore);
        //score += tscore / (terms.len() as f32);
    }
    let coverage: f32 = (cover as f32) / (terms.len() as f32);
    score *= coverage * coverage;
    // Score based on most popular category of high scoring items (narrow top results - need good already)
    // category not available at the moment
    // Grade on price & quantity matching
    //score - (price as i32) - (item.name.len() as i32)
    //print!(" / {:?} = {:?}", item.name, score);
    score
}

fn query_cache(search_term: &str) -> Option<HashMap<u32, Vec<SearchResult>>> {
    Some(QUERY_CACHE.read().ok()?.get(search_term)?.clone())
}

fn write_cache(search_term: &str, result: HashMap<u32, Vec<SearchResult>>) -> () {
    let Ok(mut map) = QUERY_CACHE.write() else {return;};
    map.insert(search_term.to_owned(), result);
}

fn query_db(search_terms: &[&str]) -> Result<HashMap<usize, HashMap<u32, Vec<SearchResult>>>> {
    // Get the database connection
    let conn: Connection = db_access::open_database()?;
    let mut result: HashMap<usize, HashMap<u32, Vec<SearchResult>>> = HashMap::new();
    for term_i in 0..search_terms.len() {
        let search_term = search_terms.get(term_i).unwrap();
        if search_term.len() < 2 {
            continue;
        }
        // Check cache
        let cache_res = query_cache(search_term);
        if cache_res.is_some() {
            result.insert(term_i, cache_res.unwrap());
        } else {
            let mut term_results: HashMap<u32, Vec<SearchResult>> = HashMap::new();
            for i in 1..=3 {
                let _shop_id = &i.to_string();
                let sql_query = "
                    SELECT id, supermarket_id, name, price, volume_size, member_price
                    FROM products p
                    WHERE p.supermarket_id = ?1
                    AND LOWER(p.name) LIKE ?2
                ";
                let search_pattern = format!("%{}%", search_term.trim().to_lowercase());    
                let mut stm = conn.prepare(sql_query)?;
                let res = stm.query_map( 
                    rusqlite::params! {i, search_pattern,},
                    |row: &rusqlite::Row<'_>|->Result<SearchResult, Error>{
                        let amt = get_unit(row.get(4).unwrap_or("ea".to_owned()));
                        let p: u32 = (row.get::<usize,f32>(3)? * 100.0) as u32;
                        let mp: Option<f32> = row.get::<usize,f32>(5).ok();
                        Ok(SearchResult {
                            item_id: row.get(0)?,
                            name: row.get(2)?,
                            price: p,
                            member_price: if mp.is_some() {(mp.unwrap() * 100.00) as u32} else {p},
                            store: match_sid_to_brand(row.get(1)?).unwrap(),
                            quantity: amt.0,
                            unit: amt.1,
                            image_url: "nada".to_owned(),
                        })
                })?;
                let shop_results: Vec<SearchResult> = res.map(|f: std::prelude::v1::Result<SearchResult, Error>|->SearchResult{f.unwrap()}).collect();
                term_results.insert(i, shop_results);
            }
            // Update cache
            write_cache(search_term, term_results.clone());
            result.insert(term_i, term_results);
        }
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
    let mulre = Regex::new(r"[0-9]+.*x.*[0-9]+.*").unwrap();
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

    fn clear_cache() {
        let Ok(mut map) = QUERY_CACHE.write() else {return;};
        map.clear();
    }

    #[test]
    fn test_result() {
        clear_cache();
        let _ = resolve(&ShoppingItemQuery {
            name: String::from("Eggs"),
            quantity: 1,
            unit: SearchUnits::EACH.to_str().to_owned(),
        }, false, false);
    }

    #[test]
    fn test_weetbix() {
        clear_cache();
        {
            let res = resolve(&ShoppingItemQuery {
                name: String::from("Weetbix"),
                quantity: 1,
                unit: SearchUnits::GRAM.to_str().to_owned(),
            }, false, false)
            .unwrap();
            assert_eq!(false, res.contains_key(&StoreBrand::Paknsave));
            assert_eq!(false, res.contains_key(&StoreBrand::Woolworths));
            assert_eq!(false, res.contains_key(&StoreBrand::Newworld));
        }
        {
            let res = resolve(&ShoppingItemQuery {
                name: String::from("Weet-Bix"),
                quantity: 1,
                unit: SearchUnits::GRAM.to_str().to_owned(),
            }, false, false)
            .unwrap();
            assert_eq!(false, res.contains_key(&StoreBrand::Paknsave));
            assert_eq!(true, res.contains_key(&StoreBrand::Woolworths));
            assert_eq!(false, res.contains_key(&StoreBrand::Newworld));
        }
    }

    #[test]
    fn test_dip() {
        clear_cache();
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Onion Soup"),
            quantity: 400,
            unit: SearchUnits::GRAM.to_str().to_owned(),
        }, false, false)
        .unwrap(); //Maggi Onion Soup
        assert_eq!("Onion Soup Mix Sachet", res.get(&StoreBrand::Paknsave).unwrap().name);
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Reduced Cream"),
            quantity: 1,
            unit: SearchUnits::MILLILITRE.to_str().to_owned(),
        }, false, false)
        .unwrap();
        assert_eq!("Reduced Cream", res.get(&StoreBrand::Paknsave).unwrap().name);
        assert_eq!("Nestle Reduced Cream Tear Tab Can 230mL", res.get(&StoreBrand::Woolworths).unwrap().name);
    }

    #[test]
    fn test_caching() {
        clear_cache();
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Onion Soup"),
            quantity: 1,
            unit: SearchUnits::GRAM.to_str().to_owned(),
        }, false, false)
        .unwrap(); //Maggi Onion Soup
        assert_eq!("Onion Soup Mix Sachet", res.get(&StoreBrand::Paknsave).unwrap().name);
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Reduced Cream"),
            quantity: 1,
            unit: SearchUnits::MILLILITRE.to_str().to_owned(),
        }, false, false)
        .unwrap();
        assert_eq!("Reduced Cream", res.get(&StoreBrand::Paknsave).unwrap().name);
        assert_eq!("Nestle Reduced Cream Tear Tab Can 230mL", res.get(&StoreBrand::Woolworths).unwrap().name);
        // Repeat test to ensure that works both times
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Onion Soup"),
            quantity: 1,
            unit: SearchUnits::GRAM.to_str().to_owned(),
        }, false, false)
        .unwrap(); //Maggi Onion Soup
        assert_eq!("Onion Soup Mix Sachet", res.get(&StoreBrand::Paknsave).unwrap().name);
        let res = resolve(&ShoppingItemQuery {
            name: String::from("Reduced Cream"),
            quantity: 400,
            unit: SearchUnits::MILLILITRE.to_str().to_owned(),
        }, false, false)
        .unwrap();
        assert_eq!("Reduced Cream", res.get(&StoreBrand::Paknsave).unwrap().name);
        assert_eq!("Nestle Reduced Cream Tear Tab Can 230mL", res.get(&StoreBrand::Woolworths).unwrap().name);
    }

    #[test]
    fn test_search() {
        assert_eq!(10, search("Milk", 10).unwrap().len());
    }

    fn simple_query(sql_query: &str) -> Vec<String> {
        let conn: Connection = db_access::open_database().unwrap();
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

    #[test]
    fn test_unit_bad() {
        println!("{:?}", get_unit("24x600mL".to_owned()));
    }

    // #[test]
    // fn test_rand() {
    //     simple_query("SELECT name FROM products p WHERE LOWER(p.name) LIKE '%Chocolate%'").into_iter()
    //         .take(500)
    //         .for_each(|x|->(){println!("{:?}", x)});
    // }
}
