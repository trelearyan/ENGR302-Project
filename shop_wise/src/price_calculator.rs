mod shopping_list_parser;
mod item_resolver;

use std::collections::HashMap;

use util::{cost::Cost, store::StoreBrand};

type StoreId = u32;
type ItemId = u32;
type RoutePlan = Vec<StoreId>;
type ShoppingPlan = HashMap<StoreId, Vec<ItemId>>;

struct ItemInfo {
    product_name: String,
    price: Cost,
    quantity: String,
}

struct CurrentPlan {
    current_shop_plan: ShoppingPlan,
    current_stores: RoutePlan,
    current_item_cost: u32,
    current_travel_cost: u32,
}

#[derive(PartialEq, Debug)]
struct BestPlan {
    best_shop_plan: ShoppingPlan,
    best_cost: u32,
}

// Mocking route_planner
#[derive(Debug, PartialEq)]
struct LocalStore {
    store_name: String,
    store_id: u32,
    brand: StoreBrand,
}

#[derive(Debug, PartialEq)]
struct LocalRoute<'a> {
    shops: Box<[&'a LocalStore]>,
    route_cost: Cost,
}

pub mod calculator {
    use std::hash::Hash;

use util::{search::{ShoppingItem, ShoppingItemQuery, unit_to_str}, store::Store};

    use crate::price_calculator::{item_resolver::resolve, *};

    ///
    /*#[must_use]
    pub fn all_possible_routes(filters: &StoreFilters) -> Box<[Route]> {
        let stores = filter_stores(&all_stores(), filters);

        (1..filters.max_store_visits)
            .flat_map(|store_visits| stores.iter().cloned().combinations(store_visits))
            .map(|stores| Route {
                shops: stores.into_boxed_slice(),
                route_cost: 0.into(),
            })
            .collect_vec()
            .into_boxed_slice()
    } */
    // Given the parsed shopping list, perform price optimisation
    pub fn calculate(list: &[ShoppingItemQuery]) {
        // Retrieve all shops - MOCK FOR NOW
        let s1 = LocalStore {
            store_name: "PakNSave Porirua".to_owned(),
            store_id: 1,
            brand: StoreBrand::Paknsave
        };
        let s2 = LocalStore {
            store_name: "Woolworths Crofton Downs".to_owned(),
            store_id: 2,
            brand: StoreBrand::Woolworths
        };
        let s3 = LocalStore {
            store_name: "NewWorld Khandallah".to_owned(),
            store_id: 3,
            brand: StoreBrand::Newworld
        };
        let all_stores = [&s1,&s2,&s3];
        let all_routes = [
            LocalRoute {
                shops: Box::new([&s1]),
                route_cost: Cost::from_cents(100),
            },
            LocalRoute {
                shops: Box::new([&s2]),
                route_cost: Cost::from_cents(100),
            },
            LocalRoute {
                shops: Box::new([&s3]),
                route_cost: Cost::from_cents(100),
            },
            LocalRoute {
                shops: Box::new([&s1, &s2]),
                route_cost: Cost::from_cents(200),
            },
            LocalRoute {
                shops: Box::new([&s1, &s3]),
                route_cost: Cost::from_cents(200),
            },
            LocalRoute {
                shops: Box::new([&s2, &s3]),
                route_cost: Cost::from_cents(200),
            },
            LocalRoute {
                shops: Box::new([&s1, &s2, &s3]),
                route_cost: Cost::from_cents(300),
            },
        ];
        // Parse all shopping items and compile short_database and helper maps
        let mut next_item_key: u32 = 0;
        let mut item_lookup: HashMap<ItemId, ItemInfo> = HashMap::new();
        let mut short_database: HashMap<StoreId, HashMap<ItemId, Cost>> = HashMap::new();
        for query in list {
            let res = resolve(query).unwrap_or(HashMap::new());
            for store in all_stores {
                if (res.contains_key(&store.store_id)) {
                    let full_item = res.get(&store.store_id).unwrap();
                    let item: ItemInfo = ItemInfo {
                        product_name: full_item.name.clone(),
                        price: Cost::from_cents(full_item.price),
                        quantity: full_item.quantity.to_string() + " " + unit_to_str(full_item.unit.clone()),
                    };
                    item_lookup.insert(next_item_key, item);
                    if (short_database.contains_key(&store.store_id)) {
                        short_database.get_mut(&store.store_id).unwrap()
                            .insert(next_item_key, Cost::from_cents(full_item.price));
                    } else {
                        let mut item_map: HashMap<ItemId, Cost> = HashMap::new();
                        item_map.insert(next_item_key, Cost::from_cents(full_item.price));
                        short_database.insert(store.store_id, item_map);
                    }
                    next_item_key += 1;
                }
            }
        }
    }

    // Perform cheapest, best, and fastest costs
    pub fn calculate_costs(
        _items: &[ShoppingItem],
        _supermarkets: &[Store],
        short_database: &HashMap<StoreId, HashMap<ItemId, i32>>) {
        /* Will call calculate cheapest, best and fastest
        *            with their respective closures and then return
        *            their results */
        
    }

    // To be replaced with closure
    fn calculate_cheapest(
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

    // To be modified to perform any search on a strictly-increasing cost function
    fn cheapest_search(
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


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::price_calculator::{self, BestPlan, ItemId, RoutePlan, StoreId};

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
        let result: Option<BestPlan> = calculate_cheapest(
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
