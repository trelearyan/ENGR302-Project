pub mod shopping_list_parser;

pub mod item_resolver;

pub mod calculator {
    use crate::{
        price_calculator::{ShoppingItem, Supermarket},
        route_planner::Route,
    };

    pub fn calculate(
        items: &[Option<ShoppingItem>],
        valid_supermarkets: &[Supermarket],
        possible_routes: &[Route],
        /* database: idfk what type this should be */
    ) {
    }
}

pub struct ShoppingItem<'a> {
    name: &'a str,
}

// Maybe this belongs in Database?
pub struct Supermarket {/* id: ??? */}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
