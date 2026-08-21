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
    current_item_cost: Cost,
    current_total_cost: Cost,
}

#[derive(PartialEq, Debug)]
pub(crate) struct BestPlan {
    best_shop_plan: ShoppingPlan,
    best_cost: Cost,
}

// Mocking route_planner
#[derive(Debug, PartialEq)]
struct LocalStore {
    store_name: String,
    store_id: u32,
    brand: StoreBrand,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LocalRoute<'a> {
    shops: Box<[&'a LocalStore]>,
    route_cost: Cost,
}

pub mod calculator {
    use std::hash::Hash;

    use bigdecimal::BigDecimal;
    use util::{search::{ShoppingItem, ShoppingItemQuery, unit_to_str}, store::Store};

    use crate::price_calculator::{item_resolver::resolve, *};

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
        let store_lookup: HashMap<StoreId, &LocalStore> = HashMap::from([
            (1, &s1),
            (2, &s2),
            (3, &s3)
        ]);
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
        let mut shop_list: Vec<ItemId> = Vec::new();
        let mut item_lookup: HashMap<ItemId, ItemInfo> = HashMap::new();
        let mut short_database: HashMap<StoreId, HashMap<ItemId, Cost>> = HashMap::new();
        for query in list {
            let res = resolve(query).unwrap_or(HashMap::new());
            for store_id in store_lookup.keys() {
                if (res.contains_key(&store_id)) {
                    let full_item = res.get(&store_id).unwrap();
                    let item: ItemInfo = ItemInfo {
                        product_name: full_item.name.clone(),
                        price: Cost::from_cents(full_item.price),
                        quantity: full_item.quantity.to_string() + " " + unit_to_str(full_item.unit.clone()),
                    };
                    item_lookup.insert(next_item_key, item);
                    if (short_database.contains_key(&store_id)) {
                        short_database.get_mut(&store_id).unwrap()
                            .insert(next_item_key, Cost::from_cents(full_item.price));
                    } else {
                        let mut item_map: HashMap<ItemId, Cost> = HashMap::new();
                        item_map.insert(next_item_key, Cost::from_cents(full_item.price));
                        short_database.insert(*store_id, item_map);
                    }
                    shop_list.push(next_item_key);
                    next_item_key += 1;
                }
            }
        }
        let res = calculate_costs(shop_list.as_ref(), &all_routes, &short_database);
    }

    // Perform cheapest, best, and fastest costs
    fn calculate_costs(
        list: &[ItemId],
        routes: &[LocalRoute],
        short_database: &HashMap<StoreId, HashMap<ItemId, Cost>>) {
        /* Will call calculate cheapest, best and fastest
        *            with their respective closures and then return
        *            their results */
        // Cheapest
        calculate_minimised(list, routes, short_database,
            |a: &Cost, b: &LocalRoute|->Cost{ return a.clone() + b.route_cost.clone(); }
        );
        // Fastest - assume route cost is proportional to time for now
        calculate_minimised(list, routes, short_database,
            |a: &Cost, b: &LocalRoute|->Cost{ return b.route_cost.clone(); }
        );
        // Best
        calculate_minimised(list, routes, short_database,
            |a: &Cost, b: &LocalRoute|->Cost{ return a.clone() + b.route_cost.clone() * 2; }
        );
    }

    // To be replaced with closure
    pub(crate) fn calculate_minimised(
        list: &[ItemId],
        routes: &[LocalRoute],
        short_database: &HashMap<StoreId, HashMap<ItemId, Cost>>,
        cost_fn: fn(ic: &Cost, lr: &LocalRoute) -> Cost,
    ) -> Option<BestPlan> {

        let mut current_shop_plan: ShoppingPlan = HashMap::new();
        let mut best_plan: Option<BestPlan> = None;

        // Run through every possible route
        'outer:
        for route in routes {
            // Clear current plan
            current_shop_plan = HashMap::new();
            // For each item pick the best store on the route
            let mut total_item: Cost = Cost::from_cents(0);
            for item in list {
                let mut best_place: Option<StoreId> = None;
                let mut best_cost: Cost = Cost::from_cents(u32::MAX);
                for shop in &route.shops {
                    let temp_cost: Option<&Cost> = short_database.get(&shop.store_id).unwrap()
                        .get(item);
                    if (temp_cost.is_some()) {
                        if (best_place.is_none() || temp_cost.unwrap() < &best_cost) {
                            best_cost = temp_cost.unwrap().clone();
                            best_place = Some(shop.store_id);
                        }
                    }
                }
                // If no store sells this item - abandon route
                if (best_place.is_none()) {
                    continue 'outer;
                }
                total_item += best_cost;
                if (current_shop_plan.contains_key(&best_place.unwrap())) {
                    current_shop_plan.get_mut(&best_place.unwrap()).unwrap().push(*item);
                } else {
                    let mut sublist: Vec<ItemId> = Vec::new();
                    sublist.push(*item);
                    current_shop_plan.insert(best_place.unwrap(), sublist);
                }
            }
            // Calculate cost and update best
            let new: Cost = cost_fn(&total_item, route);
            if (best_plan.is_none() || new < best_plan.as_ref().unwrap().best_cost) {
                best_plan = Some(BestPlan {
                    best_shop_plan: current_shop_plan,
                    best_cost: new,
                });
            }
        }

        if (best_plan.is_none()) {
            None
        } else {
            Some(best_plan.unwrap())
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use util::{cost::Cost, store::StoreBrand};
    use crate::price_calculator::calculator::calculate_minimised;
    use super::*;

    #[test]
    fn demo_test() {
        let items = [0, 1, 2];
        let supermarkets = vec![0, 1];
        let s1 = LocalStore {
            store_name: "NewWorld".to_owned(),
            store_id: 0,
            brand: StoreBrand::Newworld
        };
        let s2 = LocalStore {
            store_name: "Woolworths".to_owned(),
            store_id: 1,
            brand: StoreBrand::Woolworths
        };
        let mut routes:  Vec<LocalRoute> = Vec::new();
        routes.push(LocalRoute { shops: Box::new([&s1]), route_cost: Cost::from_cents(196) });
        routes.push(LocalRoute { shops: Box::new([&s2]), route_cost: Cost::from_cents(190) });
        routes.push(LocalRoute { shops: Box::new([&s1, &s2]), route_cost: Cost::from_cents(295) });
        let mut database: HashMap<StoreId, HashMap<ItemId, Cost>> = HashMap::new();
        let mut store0: HashMap<ItemId, Cost> = HashMap::new();
        store0.insert(0, Cost::from_cents(599));
        store0.insert(1, Cost::from_cents(720));
        store0.insert(2, Cost::from_cents(1450));
        database.insert(s1.store_id, store0);
        let mut store1: HashMap<ItemId, Cost> = HashMap::new();
        store1.insert(0, Cost::from_cents(620));
        store1.insert(1, Cost::from_cents(899));
        store1.insert(2, Cost::from_cents(1299));
        database.insert(s2.store_id, store1);
        let result: Option<BestPlan> = calculate_minimised(
            &items,
            routes.as_ref(),
            &database,
            |a: &Cost, b: &LocalRoute|->Cost{ return a.clone() + b.route_cost.clone(); },
        );
        let mut correct_shop = HashMap::new();
        correct_shop.insert(0, vec![0,1]);
        correct_shop.insert(1, vec![2]);
        let correct_result: Option<BestPlan> = Some(BestPlan {
            best_shop_plan: correct_shop,
            best_cost: Cost::from_cents(2913),
        });
        assert_eq!(correct_result, result);
    }
}
