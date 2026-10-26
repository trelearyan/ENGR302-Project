use bigdecimal::ToPrimitive;
use itertools::Itertools;
use log::logger;
use routx::{
    DEFAULT_STEP_LIMIT, Graph, KDTree, find_route_without_turn_around,
    osm::{CAR_PROFILE, FileFormat, Options},
};
use std::{
    fmt::Debug,
    iter,
    rc::Rc,
    sync::{Arc, OnceLock},
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

pub static OPTIONS: Options = routx::osm::Options {
    profile: &CAR_PROFILE,
    file_format: FileFormat::Pbf,
    bbox: [166.086_32, -47.447_61, 178.679_81, -32.975_4], // tightly bound around nz north and south islands
};
pub static GRAPH: OnceLock<(Graph, KDTree)> = OnceLock::new();

pub mod filters;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
/// An incomplete shopping plan. Note that the stop order should only include
/// stops and has not been optimised for the visit order yet.
pub struct RoutePlan {
    pub start_position: Coordinate,
    pub end_position: Coordinate,
    pub unordered_stops: Box<[Rc<Store>]>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct OptimisedRoutePlan {
    pub start_position: Coordinate,
    pub end_position: Coordinate,
    pub ordered_stops: Box<[Rc<Store>]>,
    pub expensive_calculated_route: OnceLock<Arc<[Coordinate]>>,
}

impl RoutePlan {
    #[must_use]
    pub fn optimise(&self) -> Option<OptimisedRoutePlan> {
        // let ordered_stops = iter::onceself.start_stopself.optimise_order();

        let ordered_stops: Box<[Rc<Store>]> = self
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

        log::debug!(
            "optimise called, unordered stop count = {:?}",
            self.unordered_stops.len()
        );
        log::debug!("optimise called, stop count = {:?}", ordered_stops.len());

        Some(OptimisedRoutePlan {
            start_position: self.start_position.clone(),
            end_position: self.end_position.clone(),
            ordered_stops,
            expensive_calculated_route: OnceLock::new(),
        })
    }
}

impl OptimisedRoutePlan {
    #[must_use]
    pub fn estimate_travel_cost(&self, mileage: &MileageOptions) -> Cost {
        Cost::from(mileage.clone()) * self.estimate_travel_distance().inner()
    }

    #[must_use]
    pub fn estimate_travel_time(&self, mileage: &MileageOptions) -> Duration {
        self.estimate_travel_distance().clone() / mileage.average_speed()
    }

    #[must_use]
    pub fn estimate_travel_distance(&self) -> Distance {
        self.ordered_stops
            .iter()
            .tuple_windows::<(_, _)>()
            .map(|a| a.0.location.distance_to(a.1.clone().location.clone()))
            .reduce(|acc, next| acc + next)
            .unwrap_or(Distance::default())
    }

    #[must_use]
    pub fn exact_travel_cost(&self, mileage: &MileageOptions) -> Cost {
        Cost::from(mileage.clone()) * self.exact_travel_distance().inner()
    }

    #[must_use]
    pub fn exact_travel_time(&self, mileage: &MileageOptions) -> Duration {
        self.exact_travel_distance().clone() / mileage.average_speed()
    }

    pub fn path(&self) -> Arc<[Coordinate]> {
        log::debug!(
            "path called, ordered stops count = {:?}",
            self.ordered_stops.len()
        );
        self.expensive_calculated_route
            .get_or_init(|| {
                Arc::<[Coordinate]>::from(
                    self.ordered_stops
                        .iter()
                        .map(|a| {
                            GRAPH.wait().1.find_nearest_node(
                                a.location.latitude.to_f32().unwrap(),
                                a.location.longitude.to_f32().unwrap(),
                            )
                        })
                        .tuple_windows::<(_, _)>()
                        .flat_map(|(a, b)| {
                            find_route_without_turn_around(
                                &GRAPH.wait().0,
                                a.id,
                                b.id,
                                DEFAULT_STEP_LIMIT,
                            )
                            .unwrap()
                        })
                        .map(|id| {
                            let node = GRAPH.wait().0.get_node(id).unwrap();
                            Coordinate::from_lat_long_f32(node.lat, node.lon)
                        })
                        .collect_vec(),
                )
            })
            .clone()
    }

    #[must_use]
    pub fn exact_travel_distance(&self) -> Distance {
        log::debug!(
            "exact routefind start {:?} end {:?}",
            self.start_position,
            self.end_position
        );
        let result = self
            .path()
            .iter()
            .tuple_windows::<(_, _)>()
            .map(|(a, b)| a.distance_to(b))
            .sum::<Distance>();
        log::debug!("exact travel distance call results in {result:?}");
        log::debug!("number of nodes {:?}", self.path().iter().count());
        result
    }
}

// fn find_best_route_between_two_nodes(
//     start: impl Borrow<Coordinate>,
//     end: impl Borrow<Coordinate>,
// ) -> Box<[Coordinate]> {
//     let start_node = GRAPH.wait().1.find_nearest_node(
//         start
//             .borrow()
//             .latitude
//             .to_f32()
//             .expect("lat or long to be in range of f32"),
//         start
//             .borrow()
//             .longitude
//             .to_f32()
//             .expect("lat or long to be in range of f32"),
//     );
//     let end_node = GRAPH
//         .wait()
//         .find_nearest_node(
//             end.borrow()
//                 .latitude
//                 .to_f32()
//                 .expect("lat or long to be in range of f32"),
//             end.borrow()
//                 .longitude
//                 .to_f32()
//                 .expect("lat or long to be in range of f32"),
//         )
//         .expect("end location out of range");
//     routx::find_route_without_turn_around(
//         &GRAPH.wait().0,
//         start_node.id,
//         end_node.id,
//         routx::DEFAULT_STEP_LIMIT,
//     )
//     .unwrap()
//     .iter()
//     .map(|id| {
//         let node = GRAPH
//             .wait()
//             .0
//             .get_node(*id)
//             .expect("node id to be valid after construction");
//         Coordinate::from_lat_long_f32(node.lat, node.lon)
//     })
//     .collect()
// }

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
