use crate::price_calculator::ShoppingItem;
/// Resolve a string into a ShoppingItem
/// <br>
/// Returns:
/// <br>
/// Some(ShoppingItem) if the string could be resolved as a valid
/// ShoppingItem
/// <br>
/// None if the string could not be resolved
///
/// if the 'a is confusing, thats called a lifetime. here, all it means is
/// that the returned ShoppingItem will contain a &str that 'lives' (will
/// not be destroyed) as long as the ShoppingItem itself lives. This means
/// that you will never see a ShoppingItem that contains a reference to a
/// string that has been destroyed. (this is the problem that java solves
/// via the garbage collector)
pub fn resolve<'a>(item: &'a str) -> Option<ShoppingItem<'a>> {
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
