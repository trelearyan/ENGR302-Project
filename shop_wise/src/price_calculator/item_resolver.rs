use util::search::{ShoppingItem, ShoppingItemQuery};
use std::process::Command;
use std::path::Path;
use std::fs;
// Silly rust CommandExt to stop escaping in literals
// as it allows the user of raw_arg
use std::os::windows::process::CommandExt;


/// Resolve a ShoppingItemQuery into a ShoppingItem for each store
/// <br>
/// Returns:
/// <br>
/// Some(ShoppingItem) if the string could be resolved as a valid
/// ShoppingItem
/// <br>
/// None if the string could not be resolved
pub fn resolve(item_query: &ShoppingItemQuery) -> Vec<Option<ShoppingItem>> {
    // Split item_query into search terms
    // For each supermarket (future narrow to allowed)
    // Get every item that matches some whole term of the query
    // Score each item based on whether it fits the right department,
    // Multiple words of the query (multiplicative factor), and has
    // minimal other content. This should reward "Free Range Chicken Breast"
    // over "Brand Pasta Single Snack Chicken Curry Pasta & Sauce" and
    // "Wet Cat Food Chicken Breast and Herb"
    
    // Score name based on search term matches and minimalism (might punish certain items - future)

    // Score based on most popular category of high scoring items (narrow top results - need good already)

    // Select best match - price, quantity matching

    // Return best match per store
    let csv_reply: String = demo_db("SELECT id, supermarket_id, name, price, volume_size FROM products");
    Vec::new()

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

pub fn demo_db(sql_query: &str) -> String {
    if cfg!(target_os = "windows") {
        // Should execute >cmd /C ""./src/demo_db/db.py" --query <sql>"
        // without escaping the qoutes (unliteraling my literal)
        // this is the same as typing "./src/demo_db/db.py" --query <sql>
        // into the cmd console located on shopwise/shop_wise
        Command::new("cmd")
            .raw_arg("/C \"\"./src/demo_db/db.py\" --query \"".to_owned()+sql_query+"\"\"")
            .output()
            .expect("couldn't execute query")
    } else {
        // Not yet tested on linux
        Command::new("sh")
            .arg("-c")
            .args(["./src/demo_db/db.py", "--query", "\"SELECT * FROM supermarkets\""])
            .output()
            .expect("couldn't execute query")
    };
    let path = Path::new("./src/demo_db/out.txt");
    fs::read_to_string(path)
        .expect("Should have been able to read the file")
}

#[cfg(test)]
mod tests {
    // TODO: add tests

    use util::search::ShoppingItemQuery;
    use crate::price_calculator::item_resolver::resolve;

    #[test]
    fn test_example() {
        resolve(&ShoppingItemQuery {
            name: String::from("Name"),
            quantity: 1,
            unit: String::from("ea"),
        });
    }
}
