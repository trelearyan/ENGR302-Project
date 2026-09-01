pub mod item_resolver;

use std::collections::HashMap;
use eframe::wgpu::naga::StorageFormat;
use util::{cost::Cost, store::{Store, StoreBrand}};

#[derive(Debug, PartialEq, Eq)]
pub struct ItemInfo {
    pub product_name: String,
    pub price: Cost,
    pub quantity: String,
}

#[derive(Debug)]
pub struct StorePlan {
    pub store: Store,
    pub items: Vec<ItemInfo>,
}

#[derive(Debug)]
pub struct Calculation {
    pub total_shop_cost: Cost,
    pub total_item_cost: Cost,
    pub total_travel_cost: Cost,
    pub total_time_cost: Cost,
    pub shopping_plan: Vec<StorePlan>,
}

#[derive(Debug)]
pub struct CalculationTotal {
    pub cheapest: Calculation,
    pub fastest: Calculation,
    pub best: Calculation,
}

use std::hash::Hash;

use bigdecimal::BigDecimal;
use eframe::egui::accesskit::ScrollUnit::Item;
use util::{
        search::{ShoppingItem, ShoppingItemQuery, unit_to_str},
    };

use crate::{price_calculator::item_resolver::resolve, route_planner::{all_possible_routes, all_stores, filters::{StoreFilters, filter_stores}}};
type StoreId = u32;
type ItemId = u32;
type RoutePlan = Vec<StoreId>;
type ShoppingPlan = HashMap<StoreId, Vec<ItemId>>;

#[derive(PartialEq, Debug)]
struct BestPlan {
    best_shop_plan: ShoppingPlan,
    best_cost: Cost,
    total_item_cost: Cost,
    total_travel_cost: Cost,
    total_shop_cost: Cost,
}

#[derive(Debug, PartialEq)]
struct LocalRoute {
    shops: Vec<StoreId>,
    route_travel_cost: Cost,
    route_time_cost: Cost,
}

// Given the parsed shopping list, perform price optimisation
pub fn calculate(list: &[ShoppingItemQuery], filters: &StoreFilters) -> Option<CalculationTotal> {
    let stores: &[Store] = &filter_stores(&all_stores(), filters);
    let mut store_lookup: HashMap<StoreId, Store> = HashMap::new();
    let mut i: StoreId = 0;
    for store in stores {
        store_lookup.insert(i, store.clone());
    }
    let all_routes = all_possible_routes(filters);
    // Mock route_planner route times
    let mut local_routes: Vec<LocalRoute> = Vec::new();
    for route in all_routes {
        // Calculate StoreId of Store
        let mut shops: Vec<StoreId> = Vec::new();
        for shop in route.shops {
            for i in 0..stores.len() {
                let cstore = &stores[i];
                // Mocking - identifiy stores by brand and location
                if (shop.brand == cstore.brand && shop.location == cstore.location) {
                    shops.push(i as u32);
                }
            }
        }
        local_routes.push(LocalRoute {
            shops: shops,
            route_travel_cost: route.route_cost,
            route_time_cost: Cost::from_cents(0), // Mock as 0
        })
    }
    // Parse all shopping items and compile short_database and helper maps
    let mut next_item_key: u32 = 0;
    let mut shop_list: Vec<ItemId> = Vec::new();
    let mut item_lookup: HashMap<ItemId, HashMap<StoreId, ItemInfo>> = HashMap::new();
    let mut short_database: HashMap<StoreId, HashMap<ItemId, Cost>> = HashMap::new();
    for query in list {
        let res = resolve(query).unwrap_or(HashMap::new());
        for store_id in store_lookup.keys() {
            // Mock store specific availability by just assuming same
            // among all stores within a brand
            let store_key = match store_lookup.get(store_id).unwrap().brand {
                StoreBrand::Paknsave => 1,
                StoreBrand::Woolworths => 2,
                StoreBrand::Newworld => 3,
            };
            if (res.contains_key(&store_key)) {
                let full_item = res.get(&store_key).unwrap();
                let item: ItemInfo = ItemInfo {
                    product_name: full_item.name.clone(),
                    price: Cost::from_cents(full_item.price),
                    quantity: full_item.quantity.to_string()
                        + " "
                        + unit_to_str(full_item.unit.clone()),
                };
                // Add item to item lookup table
                if (item_lookup.contains_key(&next_item_key)) {
                    item_lookup
                        .get_mut(&next_item_key)
                        .unwrap()
                        .insert(*store_id, item);
                } else {
                    let mut item_map: HashMap<ItemId, ItemInfo> = HashMap::new();
                    item_map.insert(*store_id, item);
                    item_lookup.insert(next_item_key, item_map);
                }
                // Add price to short database
                if (short_database.contains_key(&store_id)) {
                    short_database
                        .get_mut(&store_id)
                        .unwrap()
                        .insert(next_item_key, Cost::from_cents(full_item.price));
                } else {
                    let mut item_map: HashMap<ItemId, Cost> = HashMap::new();
                    item_map.insert(next_item_key, Cost::from_cents(full_item.price));
                    short_database.insert(*store_id, item_map);
                }
            }
        }
        shop_list.push(next_item_key);
        next_item_key += 1;
    }
    // Calculate best options
    let res: Option<(BestPlan, BestPlan, BestPlan)> =
        calculate_costs(shop_list.as_ref(), &local_routes, &short_database);
    // De-localise and return best plans
    if (res.is_none()) {
        None
    } else {
        let unwrapped = res.unwrap();
        let cheap: Calculation = delocalize(unwrapped.0, &item_lookup, &store_lookup);
        let fast: Calculation = delocalize(unwrapped.1, &item_lookup, &store_lookup);
        let best: Calculation = delocalize(unwrapped.2, &item_lookup, &store_lookup);
        Some(CalculationTotal {
            cheapest: cheap,
            fastest: fast,
            best: best,
        })
    }
}

fn delocalize(plan: BestPlan,
    item_lookup: &HashMap<ItemId, HashMap<StoreId, ItemInfo>>,
    store_lookup: &HashMap<StoreId, Store>) -> Calculation {
    let mut shopping_plan: Vec<StorePlan> = Vec::new();
    for i in plan.best_shop_plan.keys() {
        let mut resitem: Vec<ItemInfo> = Vec::new();
        for j in plan.best_shop_plan.get(i).unwrap() {
            let item: &ItemInfo = item_lookup.get(j).unwrap().get(i).unwrap();
            resitem.push(ItemInfo {
                product_name: item.product_name.clone(),
                price: item.price.clone(),
                quantity: item.quantity.clone(),
            });
        }
        shopping_plan.push(StorePlan {
            store: store_lookup.get(i).unwrap().clone(),
            items: resitem,
        });
    }
    Calculation {
        total_shop_cost: plan.total_shop_cost,
        total_item_cost: plan.total_item_cost,
        total_travel_cost: plan.total_travel_cost,
        total_time_cost: Cost::from_cents(0), // Mock travel time as not yet implemented
        shopping_plan: shopping_plan,
    }
}

// Perform cheapest, best, and fastest costs
fn calculate_costs(
    list: &[ItemId],
    routes: &[LocalRoute],
    short_database: &HashMap<StoreId, HashMap<ItemId, Cost>>,
) -> Option<(BestPlan, BestPlan, BestPlan)> {
    // Cheapest - lowest total cost
    let cheapest = calculate_minimised(
        list,
        routes,
        short_database,
        |a: &Cost, b: &LocalRoute| -> Cost {
            return a.clone() + b.route_travel_cost.clone();
        },
    );
    // Fastest - assume route cost is proportional to time for now
    let fastest = calculate_minimised(
        list,
        routes,
        short_database,
        |a: &Cost, b: &LocalRoute| -> Cost {
            return b.route_time_cost.clone();
        },
    );
    // Best - use route cost as time cost for now
    let best = calculate_minimised(
        list,
        routes,
        short_database,
        |a: &Cost, b: &LocalRoute| -> Cost {
            return a.clone() + b.route_travel_cost.clone() + b.route_time_cost.clone();
        },
    );
    if (cheapest.is_none() || fastest.is_none() || best.is_none()) {
        return None;
    } else {
        Some((cheapest.unwrap(), fastest.unwrap(), best.unwrap()))
    }
}

fn calculate_minimised(
    list: &[ItemId],
    routes: &[LocalRoute],
    short_database: &HashMap<StoreId, HashMap<ItemId, Cost>>,
    cost_fn: fn(ic: &Cost, lr: &LocalRoute) -> Cost,
) -> Option<BestPlan> {
    let mut current_shop_plan: ShoppingPlan = HashMap::new();
    let mut best_plan: Option<BestPlan> = None;

    // Run through every possible route
    'outer: for route in routes {
        // Clear current plan
        current_shop_plan = HashMap::new();
        // For each item pick the best store on the route
        let mut total_item: Cost = Cost::from_cents(0);
        for item in list {
            let mut best_place: Option<StoreId> = None;
            let mut best_cost: Cost = Cost::from_cents(u32::MAX);
            for shop in &route.shops {
                if (!short_database.contains_key(&shop)) {
                    continue 'outer;
                }
                let temp_cost: Option<&Cost> =
                    short_database.get(&shop).unwrap().get(item);
                if (temp_cost.is_some()) {
                    if (best_place.is_none() || temp_cost.unwrap() < &best_cost) {
                        best_cost = temp_cost.unwrap().clone();
                        best_place = Some(*shop);
                    }
                }
            }
            // If no store sells this item - abandon route
            if (best_place.is_none()) {
                continue 'outer;
            }
            total_item += best_cost;
            if (current_shop_plan.contains_key(&best_place.unwrap())) {
                current_shop_plan
                    .get_mut(&best_place.unwrap())
                    .unwrap()
                    .push(*item);
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
                total_item_cost: total_item.clone(),
                total_travel_cost: route.route_travel_cost.clone(),
                total_shop_cost: total_item + route.route_travel_cost.clone(),
            });
        }
    }

    if (best_plan.is_none()) {
        None
    } else {
        Some(best_plan.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::accesskit::Role::Search;
    use std::collections::HashMap;
    use util::{cost::Cost, distance::Distance, search::ShoppingItemQuery, store::StoreBrand};

    #[test]
    fn test_cheapest_demo() {
        let items = [0, 1, 2];
        let supermarkets = vec![0, 1];
        let mut routes: Vec<LocalRoute> = Vec::new();
        routes.push(LocalRoute {
            shops: [0].to_vec(),
            route_travel_cost: Cost::from_cents(196),
            route_time_cost: Cost::from_cents(0),
        });
        routes.push(LocalRoute {
            shops: [1].to_vec(),
            route_travel_cost: Cost::from_cents(190),
            route_time_cost: Cost::from_cents(0),
        });
        routes.push(LocalRoute {
            shops: [0,1].to_vec(),
            route_travel_cost: Cost::from_cents(295),
            route_time_cost: Cost::from_cents(0),
        });
        let mut database: HashMap<StoreId, HashMap<ItemId, Cost>> = HashMap::new();
        let mut store0: HashMap<ItemId, Cost> = HashMap::new();
        store0.insert(0, Cost::from_cents(599));
        store0.insert(1, Cost::from_cents(720));
        store0.insert(2, Cost::from_cents(1450));
        database.insert(0, store0);
        let mut store1: HashMap<ItemId, Cost> = HashMap::new();
        store1.insert(0, Cost::from_cents(620));
        store1.insert(1, Cost::from_cents(899));
        store1.insert(2, Cost::from_cents(1299));
        database.insert(1, store1);
        let result: Option<BestPlan> = calculate_minimised(
            &items,
            routes.as_ref(),
            &database,
            |a: &Cost, b: &LocalRoute| -> Cost {
                return a.clone() + b.route_travel_cost.clone();
            },
        );
        let mut correct_shop = HashMap::new();
        correct_shop.insert(0, vec![0, 1]);
        correct_shop.insert(1, vec![2]);
        let correct_result: Option<BestPlan> = Some(BestPlan {
            best_shop_plan: correct_shop,
            best_cost: Cost::from_cents(2913),
            total_item_cost: Cost::from_cents(2618),
            total_travel_cost: Cost::from_cents(295),
            total_shop_cost: Cost::from_cents(2913),
        });
        assert_eq!(correct_result, result);
    }

    #[test]
    fn test_whole() {
        let queries: &[ShoppingItemQuery] = &[ShoppingItemQuery {
            name: "Weet-Bix".to_owned(),
            quantity: 1,
            unit: "ea".to_owned(),
        }];
        let filters = StoreFilters::builder()
                    .range(Distance::from_kilometres_f64(200.0)).build();
        println!("{:?}", calculate(queries, &filters));
    }
}
