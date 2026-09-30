use itertools::Itertools;
use std::{
    fmt::Debug,
    iter,
    rc::Rc,
    time::Duration,
};

use util::{
    coordinate::Coordinate,
    cost::Cost,
    distance::Distance,
    store::{Store, StoreBrand},
};

use crate::{
    gui::transit::MileageOptions,
    route_planner::filters::{StoreFilters, filter_stores},
};

pub mod filters;

#[derive(Debug, PartialEq)]
/// An incomplete shopping plan. Note that the stop order should only include
/// stops and has not been optimised for the visit order yet.
pub struct RoutePlan {
    pub unordered_stops: Rc<[Coordinate]>,
    pub start_stop: Coordinate,
    pub end_stop: Coordinate,
    pub mileage: MileageOptions,
}

/// Complete shopping plan. Note `ordered_stops` includes the start and end stop
/// (the user location)
pub struct RoutePath {
    pub ordered_stops: Rc<[Coordinate]>,
    pub mileage: MileageOptions,
    pub travel_distance: Distance,
    pub travel_cost: Cost,
    pub travel_time: Duration,
}

impl RoutePlan {
    #[must_use]
    pub fn calculate(&self) -> RoutePath {
        // let ordered_stops = iter::onceself.start_stopself.optimise_order();

        let ordered_stops: Rc<[Coordinate]> = iter::once(&self.start_stop)
            .chain(&*self.optimise_order())
            .chain(iter::once(&self.end_stop))
            .map(std::clone::Clone::clone)
            .collect_vec()
            .into();

        let travel_distance: Distance = ordered_stops
            .array_windows::<2>()
            .map(|a| a[0].distance_to(a[1].clone()))
            .reduce(|acc, next| acc + next)
            .expect("Expected there to be more than 0 stops");

        let travel_time: Duration = travel_distance.clone() / self.mileage.average_speed();

        let travel_cost = Cost::from(self.mileage.clone()) * travel_distance.inner();

        RoutePath {
            ordered_stops,
            mileage: self.mileage.clone(),
            travel_distance,
            travel_cost,
            travel_time,
        }
    }

    fn optimise_order(&self) -> Rc<[Coordinate]> {
        // TODO: implement stop order optimisation
        self.unordered_stops.clone()
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
pub fn all_possible_routes(filters: &StoreFilters, mileage: &MileageOptions) -> Box<[RoutePath]> {
    let stores = filter_stores(&all_stores(), filters);

    (1..=filters.max_store_visits)
        .flat_map(|store_visits| stores.iter().cloned().combinations(store_visits))
        .map(|stores| RoutePlan {
            unordered_stops: stores
                .iter()
                .map(|a| a.location.clone())
                .collect_vec()
                .as_slice()
                .into(),
            start_stop: filters.location.clone(),
            end_stop: filters.location.clone(),
            mileage: mileage.clone(),
        })
        .map(|plan| plan.calculate())
        .collect_vec()
        .into_boxed_slice()
}
