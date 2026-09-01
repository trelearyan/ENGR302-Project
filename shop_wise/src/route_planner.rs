use bigdecimal::ToPrimitive;
use itertools::Itertools;
use std::fmt::{Debug, Display};

use util::{
    coordinate::Coordinate,
    cost::Cost,
    store::{Store, StoreBrand},
};

use crate::{
    input_gui::LocationState,
    route_planner::filters::{StoreFilters, filter_stores},
};

pub mod filters;

#[derive(Debug, PartialEq)]
pub struct Route {
    pub shops: Box<[Store]>,
    pub route_cost: Cost,
}

impl Route {
    /// Returns the optimal route between all of this Route's shops and
    /// (optionally) a starting and ending point. Should only be called on the
    /// final route after price calculator has narrowed down the item costs.
    pub fn optimal_route(start: Option<&LocationState>, end: Option<&LocationState>) {
        todo!();
    }

    pub fn pretty_print(&self) -> String {
        let cost = self.route_cost.clone().inner().to_f32().unwrap();
        let mut shops = String::new();
        for shop in self.shops.clone() {
            shops += format!(
                "\n        Shop({:?}, lat: {:.6}, long: {:.6})",
                shop.brand,
                shop.location.latitude.to_f32().unwrap(),
                shop.location.longitude.to_f32().unwrap()
            )
            .as_str();
        }
        format!("Route (\n    Cost: {cost:?}\n    Shops: {shops}\n)")
    }
}

// Returns every possible route the user could take between supermarkets based
// on their filters.
#[must_use]
pub fn all_stores() -> Box<[Store]> {
    Box::new([
        Store {
            brand: StoreBrand::Paknsave,
            location: Coordinate::default(),
        },
        Store {
            brand: StoreBrand::Newworld,
            location: Coordinate::default(),
        },
        Store {
            brand: StoreBrand::Woolworths,
            location: Coordinate::default(),
        },
    ])
}

// Returns every possible route the user could take between supermarkets based
// on their filters.
#[must_use]
pub fn all_possible_routes(filters: &StoreFilters) -> Box<[Route]> {
    let stores = filter_stores(&all_stores(), filters);

    (1..=filters.max_store_visits)
        .flat_map(|store_visits| stores.iter().cloned().combinations(store_visits))
        .map(|stores| Route {
            shops: stores.into_boxed_slice(),
            route_cost: 0.into(),
        })
        .collect_vec()
        .into_boxed_slice()
}
