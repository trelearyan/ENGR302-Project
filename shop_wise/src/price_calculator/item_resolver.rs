pub struct ShoppingItem; // TODO: this struct, probably belongs in Util

/// Resolve a string into a ShoppingItem
/// <br>
/// Returns:
/// <br>
/// Some(ShoppingItem) if the string could be resolved as a valid
/// ShoppingItem
/// <br>
/// None if the string could not be resolved
pub fn resolve<'a>(item: &'a str) -> Option<ShoppingItem> {
    let _shut_up_warning = item;
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
