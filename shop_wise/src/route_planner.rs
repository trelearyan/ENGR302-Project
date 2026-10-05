use itertools::Itertools;
use std::{fmt::Debug, iter, rc::Rc, time::Duration};

use util::{coordinate::Coordinate, cost::Cost, distance::Distance, store::Store};

use crate::{
    gui::transit::MileageOptions,
    route_planner::filters::{StoreFilters, filter_stores},
};

pub mod filters;
pub mod loc_resolver;

#[derive(Debug, PartialEq)]
/// An incomplete shopping plan. Note that the stop order should only include
/// stops and has not been optimised for the visit order yet.
pub struct RoutePlan {
    pub start_position: Coordinate,
    pub end_position: Coordinate,
    pub unordered_stops: Box<[Rc<Store>]>,
}

pub struct OptimisedRoutePlan {
    pub start_position: Coordinate,
    pub end_position: Coordinate,
    pub ordered_stops: Box<[Rc<Store>]>,
}

/// Complete shopping plan. Note `ordered_stops` includes the start and end stop
/// (the user location)
pub struct RoutePath {
    pub ordered_steps: Rc<[Coordinate]>,
}

impl RoutePlan {
    #[must_use]
    pub fn optimise(&self) -> Option<OptimisedRoutePlan> {
        // let ordered_stops = iter::onceself.start_stopself.optimise_order();

        let ordered_stops = self
            .unordered_stops
            .clone()
            .into_iter()
            .permutations(self.unordered_stops.len())
            .map(|stoplist| {
                (
                    stoplist.clone(),
                    iter::once(&self.start_position)
                        .chain(stoplist.iter().map(|stop| &stop.location))
                        .chain(iter::once(&self.end_position))
                        .tuple_windows::<(_, _)>() // map A,B,C,D,E to AB, BC, CD, DE
                        .map(|(a, b)| a.distance_to(b.clone())) // map AB, BC, to A -> B, B -> C,
                        .sum::<Distance>(),
                )
            })
            .min_by(|(_a_store, a_score), (_b_store, b_score)| a_score.cmp(b_score))
            .map(|(stores, _score)| stores)?
            .into();

        Some(OptimisedRoutePlan {
            start_position: self.start_position.clone(),
            end_position: self.end_position.clone(),
            ordered_stops,
        })
    }
}

impl OptimisedRoutePlan {
    pub fn estimate_travel_cost(&self, mileage: &MileageOptions) -> Cost {
        Cost::from(mileage.clone()) * self.estimate_travel_distance().inner()
    }

    pub fn estimate_travel_time(&self, mileage: &MileageOptions) -> Duration {
        self.estimate_travel_distance().clone() / mileage.average_speed()
    }

    pub fn estimate_travel_distance(&self) -> Distance {
        self.ordered_stops
            .iter()
            .tuple_windows::<(_, _)>()
            .map(|a| a.0.location.distance_to(a.1.clone().location.clone()))
            .reduce(|acc, next| acc + next)
            .expect("Expected there to be more than 0 stops")
    }
}

// Returns every possible route the user could take between supermarkets based
// on their filters.
#[must_use]
pub fn all_stores() -> Box<[Store]> {
    let stores = loc_resolver::load_store_locations().unwrap();
    stores.into_boxed_slice()
}

// Returns every possible route the user could take between supermarkets based
// on their filters.
#[must_use]
pub fn all_possible_routes(filters: &StoreFilters) -> Box<[OptimisedRoutePlan]> {
    let stores = filter_stores(&all_stores(), filters);

    (1..=filters.max_store_visits)
        .flat_map(|store_visits| stores.iter().cloned().combinations(store_visits))
        .map(|stores| RoutePlan {
            unordered_stops: stores
                .iter()
                .map(|store| Rc::new(store.clone()))
                .collect_vec()
                .into_boxed_slice(),
            start_position: filters.location.clone(),
            end_position: filters.location.clone(),
        })
        .filter_map(|plan| plan.optimise())
        .collect_vec()
        .into_boxed_slice()
}
