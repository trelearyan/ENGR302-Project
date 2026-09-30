use itertools::Itertools;
use log::logger;
use routx::AStarError;
use routx::{Graph, osm::Options};
use std::sync::LazyLock;
use std::{borrow::Borrow, fmt::Debug, iter, rc::Rc, time::Duration};

use util::{
    coordinate::Coordinate,
    cost::Cost,
    distance::Distance,
    store::Store,
};

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
}

#[must_use]
fn to_dist(coords: impl IntoIterator<Item = impl Borrow<Coordinate> + Clone>) -> Distance {
    coords
        .into_iter()
        .tuple_windows::<(_, _)>()
        .map(|(a, b)| a.borrow().distance_to(b.borrow()))
        .sum()
}

impl RoutePlan {
    /// Returns Some(RoutePath) if there is at least 1 stop in `self.unordered_stops`, None otherwise
    #[must_use]
    pub fn calculate(&self) -> Option<RoutePath> {
        let ordered_stops: Vec<_> = self
            .unordered_stops
            .iter()
            .permutations(self.unordered_stops.len()) // unordered stops -> iter of every possible stop ordering
            .map(|route_vec| {
                iter::once(self.start_stop.clone()) //                      add start and end stops to
                    .chain(route_vec.clone().iter().cloned().map(|a| a.clone())) // the start and end of each
                    .chain(iter::once(self.end_stop.clone())) //            route, also collect into vecs
                    .tuple_windows::<(_, _)>()
                    .flat_map(|(a, b)| NZ.route_search(a, b)) // path between stops
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .min_by(|iter_a, iter_b| {
                to_dist(iter_a.into_iter()).cmp(&to_dist(iter_b.into_iter())) // find minimum 
            })?;

        Some(RoutePath {
            ordered_stops: ordered_stops.into(),
            mileage: self.mileage.clone(),
        })
    }
}

impl RoutePath {
    pub fn travel_distance(&self) -> Distance {
        to_dist(self.ordered_stops.clone().into_iter())
    }
    pub fn travel_time(&self) -> Duration {
        self.travel_distance().clone() / self.mileage.average_speed()
    }
    pub fn travel_cost(&self) -> Cost {
        Cost::from(self.mileage.clone()) * self.travel_distance().inner()
    }

    /// You can also just read the field directly
    pub fn mileage(&self) -> MileageOptions {
        self.mileage.clone()
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
        .map(|plan| plan.calculate().unwrap())
        .collect_vec()
        .into_boxed_slice()
}

#[derive(Debug)]
struct GlobalGraph {
    graph: Graph,
    options: Options<'static>,
}

pub const NZ: LazyLock<GlobalGraph> = LazyLock::new(GlobalGraph::with_nz);

impl GlobalGraph {
    pub fn with_nz() -> Self {
        let mut graph = routx::Graph::new();

        let options = routx::osm::Options {
            profile: &routx::osm::CAR_PROFILE,
            file_format: routx::osm::FileFormat::Unknown,
            bbox: [166.086_32, -47.447_61, 178.679_81, -32.975_4], // tightly bound around nz north and south islands
        };

        #[cfg(feature = "include_routing")]
        {
            log::info!("Initializing graph");
            routx::osm::add_features_from_buffer(
                &mut graph,
                &options,
                include_bytes!("../../nz2-pruned.osm.pbf"),
            )
            .expect("failed to load nz2-pruned.osm.pbf");
            log::info!("Finished initializing graph");
        };

        #[cfg(not(feature = "include_routing"))]
        log::info!("include_routing feature gate disabled, routing data not initialized");

        Self { graph, options }
    }

    #[cfg(not(feature = "include_routing"))]
    pub fn route_search(
        &self,
        start_coord: Coordinate,
        end_coord: Coordinate,
    ) -> Result<RoutePlan, AStarError> {
        unimplemented!()
    }

    #[cfg(feature = "include_routing")]
    pub fn route_search(
        &self,
        start_coord: Coordinate,
        end_coord: Coordinate,
    ) -> Result<Vec<Coordinate>, AStarError> {
        use bigdecimal::num_traits::ToPrimitive;

        log::info!("find nearest start node");
        log::info!("satrt {start_coord:?}");
        let start_node = self
            .graph
            .find_nearest_node(
                start_coord
                    .latitude
                    .to_f32()
                    .expect("lat or long to be in range of f32"),
                start_coord
                    .longitude
                    .to_f32()
                    .expect("lat or long to be in range of f32"),
            )
            .expect("start location out of range");
        log::info!("find nearest end node");
        log::info!("end {end_coord:?}");
        let end_node = self
            .graph
            .find_nearest_node(
                end_coord
                    .latitude
                    .to_f32()
                    .expect("lat or long to be in range of f32"),
                end_coord
                    .longitude
                    .to_f32()
                    .expect("lat or long to be in range of f32"),
            )
            .expect("end location out of range");
        log::info!("start_node construction {start_node:?}");
        log::info!("end_node construction {end_node:?}");
        routx::find_route_without_turn_around(
            &self.graph,
            start_node.id,
            end_node.id,
            routx::DEFAULT_STEP_LIMIT,
        )
        .map(|ids| {
            ids.iter()
                .map(|id| {
                    let node = self
                        .graph
                        .get_node(*id)
                        .expect("node id to be valid after construction");
                    Coordinate::from_lat_long_f32(node.lat, node.lon)
                })
                .collect()
        })
    }
}
