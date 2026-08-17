use util::search::{ShoppingItem, ShoppingItemQuery};


/// Resolve a ShoppingItemQuery into a ShoppingItem for each store
/// <br>
/// Returns:
/// <br>
/// Some(ShoppingItem) if the string could be resolved as a valid
/// ShoppingItem
/// <br>
/// None if the string could not be resolved
pub fn resolve<'a>(item_query: &'a ShoppingItemQuery) -> Vec<Option<ShoppingItem>> {
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

    // Select best match - price, quantity


    todo!()
}

#[cfg(test)]
mod tests {
    // TODO: add tests

    #[test]
    fn test_example() {
        assert_eq!(2, 1 + 1);
    }
}
