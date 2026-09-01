pub mod item_resolver;

use std::collections::HashMap;
use util::{cost::Cost, store::StoreBrand};

pub type StoreId = u32;
pub type ItemId = u32;
pub type RoutePlan = Vec<StoreId>;
pub type ShoppingPlan = HashMap<StoreId, Vec<ItemId>>;

#[derive(Debug, PartialEq, Eq)]
pub struct ItemInfo {
    pub product_name: String,
    pub price: Cost,
    pub quantity: String,
}

#[derive(PartialEq, Debug)]
pub struct BestPlan {
    pub best_shop_plan: ShoppingPlan,
    pub best_cost: Cost,
    pub total_item_cost: Cost,
    pub total_travel_cost: Cost,
    pub total_shop_cost: Cost,
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
    route_travel_cost: Cost,
    route_time_cost: Cost,
}

#[derive(Debug, PartialEq)]
pub struct Calculation {
    pub total_shop_cost: Cost,
    pub total_item_cost: Cost,
    pub total_travel_cost: Cost,
    pub shopping_plan: HashMap<StoreId, Vec<ItemInfo>>,
}

#[derive(Debug, PartialEq)]
pub struct CalculationTotal {
    pub cheapest: Calculation,
    pub fastest: Calculation,
    pub best: Calculation,
}

pub mod calculator {
    use std::hash::Hash;

    use bigdecimal::BigDecimal;
    use eframe::egui::accesskit::ScrollUnit::Item;
    use util::{
        search::{ShoppingItem, ShoppingItemQuery, unit_to_str},
        store::Store,
    };

    use crate::price_calculator::{item_resolver::resolve, *};

    // Given the parsed shopping list, perform price optimisation
    pub fn calculate(list: &[ShoppingItemQuery]) -> Option<CalculationTotal> {
        // Retrieve all shops - MOCK FOR NOW
        let s1 = LocalStore {
            store_name: "PakNSave Kilbernie".to_owned(),
            store_id: 1,
            brand: StoreBrand::Paknsave,
        };
        let s2 = LocalStore {
            store_name: "Woolworths Cable Car Lane".to_owned(),
            store_id: 2,
            brand: StoreBrand::Woolworths,
        };
        let s3 = LocalStore {
            store_name: "NewWorld Willis Street".to_owned(),
            store_id: 3,
            brand: StoreBrand::Newworld,
        };
        let store_lookup: HashMap<StoreId, &LocalStore> =
            HashMap::from([(1, &s1), (2, &s2), (3, &s3)]);
        let all_routes = [
            LocalRoute {
                shops: Box::new([&s1]),
                route_travel_cost: Cost::from_cents(422), // 11.4 km
                route_time_cost: Cost::from_cents(1200),  // 24 min
            },
            LocalRoute {
                shops: Box::new([&s2]),
                route_travel_cost: Cost::from_cents(141), // 3.8 km
                route_time_cost: Cost::from_cents(600),   // 12 min
            },
            LocalRoute {
                shops: Box::new([&s3]),
                route_travel_cost: Cost::from_cents(111), // 3 km
                route_time_cost: Cost::from_cents(500),   // 12 min
            },
            LocalRoute {
                shops: Box::new([&s1, &s2]),
                route_travel_cost: Cost::from_cents(492), // 13.3 km
                route_time_cost: Cost::from_cents(1600),  // 32 min
            },
            LocalRoute {
                shops: Box::new([&s1, &s3]),
                route_travel_cost: Cost::from_cents(466), // 12.6 km
                route_time_cost: Cost::from_cents(1500),  // 30 min
            },
            LocalRoute {
                shops: Box::new([&s2, &s3]),
                route_travel_cost: Cost::from_cents(141), // 3.8 km
                route_time_cost: Cost::from_cents(650),   // 13 min
            },
            LocalRoute {
                shops: Box::new([&s1, &s2, &s3]),
                route_travel_cost: Cost::from_cents(492), // 13.3 km
                route_time_cost: Cost::from_cents(1650),  // 33 min
            },
        ];
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
                    StoreBrand::Newworld => 2,
                    StoreBrand::Woolworths => 3,
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
        let res: Option<(BestPlan, BestPlan, BestPlan)> =
            calculate_costs(shop_list.as_ref(), &all_routes, &short_database);
        // De-localise and return best plans
        let mut output: String = "Calculation output:\n\r".to_owned();
        if (res.is_none()) {
            output + "No result";
            return None;
        } else {
            let unwrapped = res.unwrap();
            let cheap: BestPlan = unwrapped.0;
            let mut cheap_shopping_plan: HashMap<StoreId, Vec<ItemInfo>> = HashMap::new();
            output += "- Cheapest:\n\r";
            output += &("   total cost: $".to_owned()+&cheap.total_shop_cost.to_string()+" (optimising cost "+&cheap.best_cost.to_string()+")\n");
            for i in cheap.best_shop_plan.keys() {
                let store: &&LocalStore = store_lookup.get(i).unwrap();
                let mut resitem: Vec<ItemInfo> = Vec::new();
                output += &("   At store: ".to_owned() + &store.store_name + "(id: " + &store.store_id.to_string() + ")\n");
                for j in cheap.best_shop_plan.get(i).unwrap() {
                    let item: &ItemInfo = item_lookup.get(j).unwrap().get(i).unwrap();
                    output += &("       item: ".to_owned() + &item.product_name + "(id: " + &j.to_string() + ") - "+&item.price.to_string()+"\n");
                    resitem.push(ItemInfo {
                        product_name: item.product_name.clone(),
                        price: item.price.clone(),
                        quantity: item.quantity.clone(),
                    });
                }
                cheap_shopping_plan.insert(*i, resitem);
            }
            let cheapres = Calculation {
                total_shop_cost: cheap.total_shop_cost,
                total_item_cost: cheap.total_item_cost,
                total_travel_cost: cheap.total_travel_cost,
                shopping_plan: cheap_shopping_plan,
            };
            let fast: BestPlan = unwrapped.1;
            let mut fast_shopping_plan: HashMap<StoreId, Vec<ItemInfo>> = HashMap::new();
            output += "\n- Fastest:\n";
            output += &("   total cost: $".to_owned()+&fast.total_shop_cost.to_string()+" (optimising cost "+&fast.best_cost.to_string()+")\n");
            for i in fast.best_shop_plan.keys() {
                let store: &&LocalStore = store_lookup.get(i).unwrap();
                let mut resitem: Vec<ItemInfo> = Vec::new();
                output += &("   At store: ".to_owned() + &store.store_name + "(id: " + &store.store_id.to_string() + ")\n");
                for j in fast.best_shop_plan.get(i).unwrap() {
                    let item: &ItemInfo = item_lookup.get(j).unwrap().get(i).unwrap();
                    output += &("       item: ".to_owned() + &item.product_name + "(id: " + &j.to_string() + ") - "+&item.price.to_string()+"\n");
                    resitem.push(ItemInfo {
                        product_name: item.product_name.clone(),
                        price: item.price.clone(),
                        quantity: item.quantity.clone(),
                    });
                }
                fast_shopping_plan.insert(*i, resitem);
            }
            let fastres = Calculation {
                total_shop_cost: fast.total_shop_cost,
                total_item_cost: fast.total_item_cost,
                total_travel_cost: fast.total_travel_cost,
                shopping_plan: fast_shopping_plan,
            };
            let best: BestPlan = unwrapped.2;
            let mut best_shopping_plan: HashMap<StoreId, Vec<ItemInfo>> = HashMap::new();
            output += "\n- Best:\n";
            output += &("   total cost: $".to_owned()+&best.total_shop_cost.to_string()+" (optimising cost "+&best.best_cost.to_string()+")\n");
            for i in best.best_shop_plan.keys() {
                let store: &&LocalStore = store_lookup.get(i).unwrap();
                let mut resitem: Vec<ItemInfo> = Vec::new();
                output += &("   At store: ".to_owned() + &store.store_name + "(id: " + &store.store_id.to_string() + ")\n");
                for j in best.best_shop_plan.get(i).unwrap() {
                    let item: &ItemInfo = item_lookup.get(j).unwrap().get(i).unwrap();
                    output += &("       item: ".to_owned() + &item.product_name + "(id: " + &j.to_string() + ") - "+&item.price.to_string()+"\n");
                    resitem.push(ItemInfo {
                        product_name: item.product_name.clone(),
                        price: item.price.clone(),
                        quantity: item.quantity.clone(),
                    });
                }
                best_shopping_plan.insert(*i, resitem);
            }
            let bestres = Calculation {
                total_shop_cost: best.total_shop_cost,
                total_item_cost: best.total_item_cost,
                total_travel_cost: best.total_travel_cost,
                shopping_plan: best_shopping_plan,
            };
            log::info!("{}", output);
            Some(CalculationTotal {
                cheapest: cheapres,
                fastest: fastres,
                best: bestres,
            })
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
        'outer: for route in routes {
            // Clear current plan
            current_shop_plan = HashMap::new();
            // For each item pick the best store on the route
            let mut total_item: Cost = Cost::from_cents(0);
            for item in list {
                let mut best_place: Option<StoreId> = None;
                let mut best_cost: Cost = Cost::from_cents(u32::MAX);
                for shop in &route.shops {
                    if (!short_database.contains_key(&shop.store_id)) {
                        continue 'outer;
                    }
                    let temp_cost: Option<&Cost> =
                        short_database.get(&shop.store_id).unwrap().get(item);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::price_calculator::calculator::{calculate, calculate_minimised};
    use eframe::egui::accesskit::Role::Search;
    use std::collections::HashMap;
    use util::{cost::Cost, search::ShoppingItemQuery, store::StoreBrand};

    #[test]
    fn test_cheapest_demo() {
        let items = [0, 1, 2];
        let supermarkets = vec![0, 1];
        let s1 = LocalStore {
            store_name: "NewWorld".to_owned(),
            store_id: 0,
            brand: StoreBrand::Newworld,
        };
        let s2 = LocalStore {
            store_name: "Woolworths".to_owned(),
            store_id: 1,
            brand: StoreBrand::Woolworths,
        };
        let mut routes: Vec<LocalRoute> = Vec::new();
        routes.push(LocalRoute {
            shops: Box::new([&s1]),
            route_travel_cost: Cost::from_cents(196),
            route_time_cost: Cost::from_cents(0),
        });
        routes.push(LocalRoute {
            shops: Box::new([&s2]),
            route_travel_cost: Cost::from_cents(190),
            route_time_cost: Cost::from_cents(0),
        });
        routes.push(LocalRoute {
            shops: Box::new([&s1, &s2]),
            route_travel_cost: Cost::from_cents(295),
            route_time_cost: Cost::from_cents(0),
        });
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
        println!("{:?}", calculate(queries));
    }
}
