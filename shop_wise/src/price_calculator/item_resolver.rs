use util::search::{ShoppingItem, ShoppingItemQuery};


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
    // "wet cat food chicken breast and herb"
    
    // Score name based on search term matches and minimalism (might punish certain items - future)

    // Score based on most popular category of high scoring items (narrow top results - need good already)

    // Select best match - price, quantity matching

    // Return best match per store
    todo!()

    // Example search "Vegemite",1,"ea" -> return cheapest vegemite at each store, with single item and price
    // Complications (includes but is not limited to)
    // 1) "Vogel's" contains an apostrophe which is not a valid character in a sql search
    // and may not be present in the database name for the item -> no results even when clear name used
    // 2) "Butter" -> "Butter Flavoured Popcorn". "X Flavoured" almost never the right result when single
    // X search term used, but next to impossible to filter out all the variations of such, dilutes both
    // department specificity and price selection
    // 3) "Chicken Breast",1,"l" "l" is not a valid quantity of chicken -> how to handle it.
    // Sometimes this is more reasonable like searching for 3L of water, but some things are sold by the
    // pack and this makes conversion difficult. Don't want to select $48 pack of canned sparkling water,
    // because couldn't resolve the $4 12 pack of bottled water when user search query contains a volume
    // (or weight)
    // 4) "Eggs",6,"ea" -> "Countdown eggs half dozen barn size 6","6pk". need to parse the quantity
    // information which will be different for every chain and type and compare to the users queried amount
    // (e.g. what if quanity was 1ea, cannot possible try to obtain info from name, i.e. parsing "half dozen")
}

#[cfg(test)]
mod tests {
    // TODO: add tests

    #[test]
    fn test_example() {
        assert_eq!(2, 1 + 1);
    }
}
