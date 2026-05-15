pub mod shopping_list_parser;

pub mod item_resolver;

pub mod calculator {
    use std::collections::HashMap;
    use crate::{ShoppingItem, Supermarket};


    type StoreId = i32;
    type ItemId = i32;
    type StoreSet = Vec<StoreId>;
    type RoutePlan = Vec<StoreId>;
    type ShoppingPlan = HashMap<StoreId, Vec<ItemId>>;

    struct CurrentPlan {
        current_shop_plan: ShoppingPlan,
        current_stores: RoutePlan,
        current_item_cost: i32,
        current_travel_cost: i32,
    }

    struct BestPlan {
        best_shop_plan: ShoppingPlan,
        best_cost: i32,
    }

    pub fn calculate(
        items: &[ShoppingItem],
        supermarkets: &[Supermarket],
        database: &HashMap<StoreId, HashMap<ItemId, i32>>) {
            // Will call calculate cheapest
        }

    pub fn calculate_cheapest(
        items: &[ItemId],
        supermarkets: &[StoreId],
        routes: &HashMap<RoutePlan, i32>,
        database: &HashMap<StoreId, HashMap<ItemId, i32>>
    ) -> Option<BestPlan> {

        let mut current_plan = CurrentPlan {
            current_shop_plan: HashMap::new(),
            current_stores: Vec::new(),
            current_item_cost: 0,
            current_travel_cost: 0,
        };

        let mut best_plan = BestPlan {
            best_shop_plan: HashMap::new(),
            best_cost: i32::MAX,
        };

        cheapest_search(0,
            items,
            database,
            routes,
            &mut current_plan,
            &mut best_plan,
        );

        if best_plan.best_cost == i32::MAX {
            // Panik! Couldn't buy all the items
            return None;
        }
        return Some(best_plan);
    }

    pub fn cheapest_search(
        item_index: usize,
        items: &[ItemId],
        database: &HashMap<StoreId, HashMap<ItemId, i32>>,
        routes: &HashMap<RoutePlan, i32>,
        current_plan: &mut CurrentPlan,
        best_plan: &mut BestPlan,
    ) {
        // Base case
        if item_index >= items.len() {
            if current_plan.current_item_cost + current_plan.current_travel_cost < best_plan.best_cost {
                best_plan.best_cost = current_plan.current_item_cost + current_plan.current_travel_cost;
                best_plan.best_shop_plan = current_plan.current_shop_plan.clone();
            }
            return;
        }
        // Retrieve item we want to add
        let item = &items[item_index];
        // Run through every possible store to purchase this item at
        for (store_id, shop_pricings) in database {
            // Check if this supermarket sells that item, otherwise skip
            if !shop_pricings.contains_key(item)  {
                continue;
            }

            let mut added_store: bool = false;
            let mut store_index: usize = 0;
            let old_travel_cost: i32 = current_plan.current_travel_cost;
            // If store not on route currently, add it
            if !current_plan.current_shop_plan.contains_key(store_id) {
                added_store = true;
                // Keep current_stores sorted, by using binary search and insertion
                store_index = match current_plan.current_stores.binary_search(*store_id) {
                    Ok(i) | Err(i) => i,
                };
                current_plan.current_stores.insert(store_index, *store_id);
                current_plan.current_travel_cost = *routes.get(&current_plan.current_stores).unwrap();
            }
            // Update current best_plan to include this decision
            current_plan.current_shop_plan
                .entry(*store_id)
                .or_insert_with(Vec::new)
                .push(*item);
            // Update current cost with item pricing
            let price = *shop_pricings.get(item).unwrap();
            current_plan.current_item_cost += price;

            // Cull recursions if we already know sub-optimal (price strictly increases)
            if current_plan.current_item_cost + current_plan.current_travel_cost < best_plan.best_cost {
                // Recurse
                cheapest_search(
                    item_index+1,
                    items,
                    database,
                    routes,
                    current_plan,
                    best_plan,
                );
            }
            // Rollback to previous state to continue search
            current_plan.current_item_cost -= price;
            if added_store {
                current_plan.current_stores.remove(store_index);
                current_plan.current_travel_cost = old_travel_cost;
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct ShoppingItem<'a> {
    id: ItemId,
    name: &'a str,
}

#[derive(Hash, Eq, PartialEq)]
pub struct Supermarket<'a> {
    id: StoreId,
    brand: &'a str
}

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
