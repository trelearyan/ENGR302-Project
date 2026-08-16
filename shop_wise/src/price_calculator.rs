mod shopping_list_parser;
mod item_resolver;

use std::collections::HashMap;

pub type StoreId = i32;
pub type ItemId = i32;
pub type RoutePlan = Vec<StoreId>;
pub type ShoppingPlan = HashMap<StoreId, Vec<ItemId>>;

pub struct CurrentPlan {
    current_shop_plan: ShoppingPlan,
    current_stores: RoutePlan,
    current_item_cost: i32,
    current_travel_cost: i32,
}

#[derive(PartialEq, Debug)]
pub struct BestPlan {
    best_shop_plan: ShoppingPlan,
    best_cost: i32,
}

//pub mod shopping_list_parser;

//pub mod item_resolver;

pub mod calculator {
    use crate::price_calculator::*;
    pub fn calculate(
        _items: &[ShoppingItem],
        _supermarkets: &[Supermarket],
        _database: &HashMap<StoreId, HashMap<ItemId, i32>>) {
        /* Will call calculate cheapest, best and fastest
         *            with their respective closures and then return
         *            their results */
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
                            supermarkets,
                            database,
                            routes,
                            &mut current_plan,
                            &mut best_plan,
            );

            if best_plan.best_cost == i32::MAX {
                /* Couldn't buy all the items and return None,
                 *            need to work out how to deal with certain errors
                 *            and some of this needs to be done in a pre-check */
                return None;
            }
            return Some(best_plan);
        }

        pub fn cheapest_search(
            item_index: usize,
            items: &[ItemId],
            supermarkets: &[StoreId],
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
                // Check if we can shop at this supermarket
                if !supermarkets.contains(&store_id) {
                    continue;
                }
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
                    store_index = match current_plan.current_stores.binary_search(store_id) {
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
                        supermarkets,
                        database,
                        routes,
                        current_plan,
                        best_plan,
                    );
                }
                // Rollback to previous state to continue search
                current_plan.current_item_cost -= price;
                if let Some(items) = current_plan.current_shop_plan.get_mut(store_id) {
                    items.pop();
                }
                if added_store {
                    current_plan.current_shop_plan.remove(store_id);
                    current_plan.current_stores.remove(store_index);
                    current_plan.current_travel_cost = old_travel_cost;
                }
            }
        }
}

pub struct ShoppingItem<'a> {
    name: &'a str,
}

// Maybe this belongs in Database?
pub struct Supermarket<'a> {
    name: &'a str,
    id: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_test() {
        let items = vec![0, 1, 2];
        let supermarkets = vec![0, 1];
        let mut routes:  HashMap<RoutePlan, i32> = HashMap::new();
        routes.insert(vec![], 0);
        routes.insert(vec![0], 196);
        routes.insert(vec![1], 190);
        routes.insert(vec![0,1],295);
        let mut database: HashMap<StoreId, HashMap<ItemId, i32>> = HashMap::new();
        let mut store0: HashMap<ItemId, i32> = HashMap::new();
        store0.insert(0, 599);
        store0.insert(1, 720);
        store0.insert(2, 1450);
        database.insert(0, store0);
        let mut store1: HashMap<ItemId, i32> = HashMap::new();
        store1.insert(0, 620);
        store1.insert(1, 899);
        store1.insert(2, 1299);
        database.insert(1, store1);
        let result: Option<BestPlan> = calculator::calculate_cheapest(
            &items,
            &supermarkets,
            &routes,
            &database
        );
        let mut correct_shop = HashMap::new();
        correct_shop.insert(0, vec![0,1]);
        correct_shop.insert(1, vec![2]);
        let correct_result: Option<BestPlan> = Some(BestPlan {
            best_shop_plan: correct_shop,
            best_cost: 2913,
        });
        assert_eq!(result,correct_result);
    }
}
