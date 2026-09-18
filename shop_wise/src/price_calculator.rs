use std::collections::HashMap;
use std::time::Duration;
use itertools::Itertools;
use util::distance::Distance;
use util::{cost::Cost, store::{Store, StoreBrand}};
use bigdecimal::ToPrimitive;
use util::{search::{ShoppingItem, ShoppingItemQuery}};
use crate::{
    gui::transit::MileageOptions,
    price_calculator::item_resolver::resolve,
    route_planner::{
        RoutePath, all_possible_routes, all_stores,
        filters::{StoreFilters, filter_stores},
    },
};

// Public types

pub mod item_resolver;
pub mod results;

#[derive(Debug)]
pub struct StorePlan {
    pub store: Store,
    pub items: Vec<ShoppingItem>,
}

#[derive(Debug)]
pub enum MessageType {
    NOTICE,
    WARNING,
}

#[derive(Debug)]
pub struct Message {
    pub msg_type: MessageType,
    pub msg: String,
}

#[derive(Debug, PartialEq)]
pub enum CalculationErrorType {
    InternalError,
    CouldNotResolveItem,
    NoPlanFound,
    CommonItemFailure,
}

#[derive(Debug, PartialEq)]
pub struct CalculationError {
    pub err_type: CalculationErrorType,
    pub err_msg: String,
}

impl CalculationError {
    #[must_use]
    pub fn pretty(&self) -> String {
        match self.err_type {
            CalculationErrorType::InternalError => "Unknown Error",
            CalculationErrorType::CouldNotResolveItem => "Item Resolution Error: ",
            CalculationErrorType::NoPlanFound => "No Complete Shopping Route Found: ",
            CalculationErrorType::CommonItemFailure => "No Shopping Route Found - Common Item cause of Failure: ",
        }.to_owned() + ": " + &self.err_msg
    }
}

#[derive(Debug)]
pub struct Calculation {
    pub total_shop_cost: Cost,
    pub total_item_cost: Cost,
    pub total_travel_cost: Cost,
    pub total_time: Duration,
    pub total_dist: Distance,
    pub shopping_plan: Vec<StorePlan>,
    pub msg_log: Vec<Message>,
}

#[derive(Debug)]
pub struct CalculationTotal {
    pub cheapest: Calculation,
    pub fastest: Calculation,
    pub best: Calculation,
}

// Private types

type StoreId = usize;
type ItemId = usize;
type RouteId = usize;
type ShoppingPlan = Vec<StoreId>;

#[derive(PartialEq, Debug)]
struct BestPlan {
    best_shop_plan: ShoppingPlan,
    best_cost: u32,
    total_item_cost: u32,
    total_travel_cost: u32,
    total_shop_cost: u32,
    total_time: u64,
    route_id: RouteId,
}

#[derive(Debug, PartialEq)]
struct LocalRoute {
    route_id: RouteId,
    shops: Vec<StoreId>,
    route_travel_cost: u32,
    route_time: u64,
}

// Given the parsed shopping list, perform price optimisation
#[must_use]
pub fn calculate(
    shop_list: &[ShoppingItemQuery],
    filters: &StoreFilters,
    mileage: &MileageOptions,
) -> Result<CalculationTotal, CalculationError> {
    // Get all visitable stores, position in Vec is local ID (StoreId)
    let stores: Vec<Store> = filter_stores(&all_stores(), filters)
        .iter()
        .map(|a: &Store| -> Store { a.to_owned() })
        .collect();
    // Fetch all (2^n-1) routes for the n stores in range
    let routes: Box<[RoutePath]> = all_possible_routes(filters, mileage);
    let mut local_routes: Vec<LocalRoute> = Vec::new();
    for route_id in 0..routes.len() {
        let route: &RoutePath = routes.get(route_id).unwrap();
        // Calculate set of StoreIds for visited stores
        let mut visited: Vec<RouteId> = Vec::new();
        for stop in route.ordered_stops.iter() {
            let mut found_id: Option<StoreId> = None;
            for i in 0..stores.len() {
                let store = &stores[i];
                if store.location == *stop {
                    found_id = Some(i);
                    break;
                }
            }
            if found_id.is_none() {
                continue;
            }
            visited.push(found_id.unwrap());
        }
        // Encode route into LocalRoute
        local_routes.push(LocalRoute {
            shops: visited,
            route_travel_cost: route
                .travel_cost
                .clone()
                .round()
                .inner()
                .with_scale(2)
                .to_u32()
                .unwrap(),
            route_time: route.travel_time.as_secs(),
            route_id,
        });
    }
    // Encode necessary data into a short_database Price indexed by ItemId, indexed by StoreId
    let mut short_database: Vec<Vec<Option<u32>>> = Vec::new();
    for _i in 0..stores.len() {
        short_database.push(Vec::new());
    }
    // Keep shopping items for returning to output_gui
    let mut item_lookup: Vec<HashMap<StoreId, ShoppingItem>> = Vec::new();
    for item_key in 0..shop_list.len() {
        item_lookup.push(HashMap::new());
        // Resolve query
        let query: &ShoppingItemQuery = shop_list.get(item_key).unwrap();
        let Some(mut res) = resolve(query) else {
            return Err(CalculationError {
                err_type: CalculationErrorType::CouldNotResolveItem,
                err_msg: "Could not find item ".to_owned() + &query.name,
            })};
        // Record resolution at each visited store
        for store_id in 0..stores.len() {
            // Mock store specific availability by just assuming same
            // among all stores within a brand
            let global_store_key = match stores.get(store_id).unwrap().brand {
                StoreBrand::Paknsave => 1,
                StoreBrand::Woolworths => 2,
                StoreBrand::Newworld => 3,
            };
            // If this store has an entry for the item add it
            if res.contains_key(&global_store_key) {
                let Some(item) = res.remove(&global_store_key) else {
            return Err(CalculationError {
                err_type: CalculationErrorType::CouldNotResolveItem,
                err_msg: "Could not find item ".to_owned() + &shop_list.get(item_key).unwrap().name,
            })};
                // Convert to cents for efficient copying
                let price = item
                    .price
                    .clone()
                    .round()
                    .inner()
                    .with_scale(2)
                    .to_u32()
                    .unwrap();
                // Add item to item lookup table
                item_lookup.get_mut(item_key).unwrap().insert(store_id, item);
                // Add price to short database
                short_database.get_mut(store_id).unwrap().push(Some(price));
            } else {
                // Don't add price to short database
                short_database.get_mut(store_id).unwrap().push(None);
            }
        }
    }
    // Calculate minimising for combined item and travel cost
    let cheap = delocalise(
        calculate_minimised(
            shop_list.len(),
            &local_routes,
            &short_database,
            |a: &u32, b: &LocalRoute| -> u32 { a + b.route_travel_cost },
            &shop_list
        )?,
        &item_lookup,
        &stores,
        &routes,
    );
    // Calculate minimising only for travel time
    let fast = delocalise(
        calculate_minimised(
            shop_list.len(),
            &local_routes,
            &short_database,
            |_a: &u32, b: &LocalRoute| -> u32 { b.route_time as u32 },
            &shop_list
        )?,
        &item_lookup,
        &stores,
        &routes,
    );
    // Calculate minimising for total cost, with a $30 hourly rate
    let best = delocalise(
        calculate_minimised(
            shop_list.len(),
            &local_routes,
            &short_database,
            |a: &u32, b: &LocalRoute| -> u32 {
                // Assume people would drive an hour to save 30 dollars for now (5/6 cents per second)
                a + b.route_travel_cost + ((5 * b.route_time) / 6) as u32
            },
            &shop_list
        )?,
        &item_lookup,
        &stores,
        &routes,
    );
    // Return the result
    Ok(CalculationTotal {
        cheapest: cheap,
        fastest: fast,
        best,
    })
}

fn delocalise(
    plan: (BestPlan, Vec<Message>),
    item_lookup: &Vec<HashMap<StoreId, ShoppingItem>>,
    stores: &Vec<Store>,
    routes: &Box<[RoutePath]>,
) -> Calculation {
    // Compile shopping plan into Vec<StorePlan>
    let mut shopping_plan: Vec<StorePlan> = Vec::new();
    for shop_id in 0..stores.len() {
        let mut items: Vec<ShoppingItem> = Vec::new();
        for item_id in 0..item_lookup.len() {
            // If this item was brought at the cuurent store
            if *plan.0.best_shop_plan.get(item_id).unwrap() == shop_id {
                let item: &ShoppingItem =
                    item_lookup.get(item_id).unwrap().get(&(shop_id)).unwrap();
                items.push(ShoppingItem {
                    name: item.name.clone(),
                    quantity: item.quantity,
                    unit: item.unit.clone(),
                    price: item.price.clone(),
                    store: item.store.clone(),
                });
            }
        }
        if !items.is_empty() {
            shopping_plan.push(StorePlan {
                store: stores.get(shop_id).unwrap().clone(),
                items,
            });
        }
    }
    let rp: &RoutePath = routes.get(plan.0.route_id).unwrap();
    Calculation {
        total_shop_cost: Cost::from_cents(plan.0.total_shop_cost),
        total_item_cost: Cost::from_cents(plan.0.total_item_cost),
        total_travel_cost: Cost::from_cents(plan.0.total_travel_cost),
        total_time: Duration::from_secs(plan.0.total_time),
        total_dist: rp.travel_distance.clone(),
        shopping_plan,
        msg_log: plan.1,
        //route: rp,
    }
}

fn calculate_minimised(
    list: ItemId,
    routes: &[LocalRoute],
    short_database: &[Vec<Option<u32>>],
    cost_fn: fn(ic: &u32, lr: &LocalRoute) -> u32,
    list_context: &[ShoppingItemQuery]
) -> Result<(BestPlan, Vec<Message>), CalculationError> {
    let mut current_shop_plan: ShoppingPlan;
    let mut best_plan: Option<BestPlan> = None;
    // Keeps track of which items were missed at ALL failing routes
    let mut all_missed_items: Option<Vec<ItemId>> = None;
    let mut or_missed_items: Option<Vec<ItemId>> = None;

    // Run through every possible route
    for route in routes { //'outer: - see below
        // Clear current plan
        current_shop_plan = Vec::new();
        // For each item pick the best store on the route
        let mut total_item: u32 = 0;
        let mut missed_items: Vec<ItemId> = Vec::new();
        for item in 0..list {
            let mut best_place: Option<StoreId> = None;
            let mut best_cost: u32 = u32::MAX;
            for shop in &route.shops {
                let temp_cost: Option<u32> = *short_database.get(*shop).unwrap().get(item).unwrap();
                if (temp_cost.is_some()) && (best_place.is_none() || temp_cost.unwrap() < best_cost) {
                    best_cost = temp_cost.unwrap();
                    best_place = Some(*shop);
                }
            }
            // If no store sells this item - abandon route and skip to next
            if best_place.is_none() {
                // For logging purposes
                missed_items.push(item);
                continue; // Could be 'outer - except for detailed error purposes
            }
            total_item += best_cost;
            current_shop_plan.push(best_place.unwrap());
        }
        // If an item was missed
        if !&missed_items.is_empty() {
            // AND it with the missed items list, to keep track of failing items
            if all_missed_items.is_none() {
                all_missed_items = Some(missed_items.clone());
            } else {
                let mut removed: usize = 0;
                for item_i in 0..all_missed_items.as_ref().unwrap().len() {
                    let value = all_missed_items.as_ref().unwrap().get(item_i).unwrap();
                    if !AsRef::<Vec<ItemId>>::as_ref(&missed_items).contains(&value) {
                        all_missed_items.as_mut().unwrap().remove(item_i-removed);
                        removed += 1;
                    }
                }
            }
            // OR it with the missed items list, to keep track of failing items
            if or_missed_items.is_none() {
                or_missed_items = Some(missed_items);
            } else {
                for item_i in 0..missed_items.len() {
                    let value = missed_items.get(item_i).unwrap();
                    if or_missed_items.as_ref().unwrap().contains(&value) {
                        or_missed_items.as_mut().unwrap().push(*value);
                    }
                }
            }
            // Abandon this route and move to the next
            continue;
        }
        // Calculate cost and update best
        let new: u32 = cost_fn(&total_item, route);
        if best_plan.is_none() || new < best_plan.as_ref().unwrap().best_cost {
            best_plan = Some(BestPlan {
                best_shop_plan: current_shop_plan,
                best_cost: new,
                total_item_cost: total_item,
                total_travel_cost: route.route_travel_cost,
                total_shop_cost: total_item + route.route_travel_cost,
                total_time: route.route_time,
                route_id: route.route_id,
            });
        }
    }
    if best_plan.is_none() {
        return if all_missed_items.as_ref().is_none() {
            Err(CalculationError {
                err_type: CalculationErrorType::InternalError,
                err_msg: "An internal error cause the plan search to fail".to_owned(),
            })
        } else {
            let all_missed = all_missed_items.unwrap();
            if all_missed.is_empty() {
                let info = or_missed_items.unwrap().iter()
                    .map(|i: &ItemId|->String{ return list_context.get(*i).unwrap().name.clone();})
                    .join(", ");
                Err(CalculationError {
                    err_type: CalculationErrorType::NoPlanFound,
                    err_msg: " Reason too complex to state. NOTE: Items ".to_owned() + &info +
                    &" were absent from one or more stores in the search.",
                })
            } else {
                let err_msg_content: String  = all_missed.iter()
                    .map(|i: &ItemId|->String{ return list_context.get(*i).unwrap().name.clone(); })
                    .join(", ");
                Err(CalculationError {
                    err_type: CalculationErrorType::CommonItemFailure,
                    err_msg: "No route was able to fulfill the following items ".to_owned() + &err_msg_content,
                })
        }}
    }
    let log_msg_content: Vec<Message> = or_missed_items.unwrap().iter()
        .map(|i: &ItemId|->Message{ return Message {
            msg_type: MessageType::NOTICE,
            msg: "Item ".to_owned() +
                &list_context.get(*i).unwrap().name +
                &" was absent from one or more stores in the search.",
        };})
        .collect::<Vec<Message>>();
    Ok((best_plan.unwrap(), log_msg_content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cheapest_demo() {
        assert_eq!(3, 1 + 2);
        let items = [0, 1, 2];
        let mut routes: Vec<LocalRoute> = Vec::new();
        routes.push(LocalRoute {
            shops: [0].to_vec(),
            route_travel_cost: 196,
            route_time: 0,
            route_id: 0,
        });
        routes.push(LocalRoute {
            shops: [1].to_vec(),
            route_travel_cost: 190,
            route_time: 0,
            route_id: 1,
        });
        routes.push(LocalRoute {
            shops: [0, 1].to_vec(),
            route_travel_cost: 295,
            route_time: 0,
            route_id: 2,
        });
        let mut database: Vec<Vec<Option<u32>>> = Vec::new();
        let mut store0: Vec<Option<u32>> = Vec::new();
        store0.push(Some(599));
        store0.push(Some(720));
        store0.push(Some(1450));
        database.push(store0);
        let mut store1: Vec<Option<u32>> = Vec::new();
        store1.push(Some(620));
        store1.push(Some(899));
        store1.push(Some(1299));
        database.push(store1);
        let result = calculate_minimised(
            items.len(),
            routes.as_ref(),
            &database,
            |a: &u32, b: &LocalRoute| -> u32 {
                return a + b.route_travel_cost;
            },
            &Vec::new(),
        );
        let correct_result= BestPlan {
            best_shop_plan: vec![0, 0, 1],
            best_cost: 2913,
            total_item_cost: 2618,
            total_travel_cost: 295,
            total_shop_cost: 2913,
            total_time: 0,
            route_id: 2,
        };
        assert!(result.is_ok());
        assert_eq!(correct_result, result.unwrap().0);
    }
}
