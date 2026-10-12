use bigdecimal::BigDecimal;
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

use crate::{gui::transit::MileageOptions, route_planner::filters::StoreFilters};

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

        // the mileage rate is per kilometre; Distance::inner() is in metres
        let travel_cost = Cost::from(self.mileage.clone())
            * (travel_distance.inner() / BigDecimal::from(1000));

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

/// Returns every route the user could take between `stores` (already filtered), visiting
/// up to `filters.max_store_visits` of them. Each route is paired with the indices into
/// `stores` of the stores it visits, so callers know which store is which without
/// matching locations (several stores can share a location).
#[must_use]
pub fn all_possible_routes(
    stores: &[Store],
    filters: &StoreFilters,
    mileage: &MileageOptions,
) -> Vec<(Vec<usize>, RoutePath)> {
    (1..=filters.max_store_visits)
        .flat_map(|store_visits| (0..stores.len()).combinations(store_visits))
        .map(|visited| {
            let path = RoutePlan {
                unordered_stops: visited
                    .iter()
                    .map(|&i| stores[i].location.clone())
                    .collect_vec()
                    .as_slice()
                    .into(),
                start_stop: filters.location.clone(),
                end_stop: filters.location.clone(),
                mileage: mileage.clone(),
            }
            .calculate();
            (visited, path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_planner::filters::StoreFilters;

    fn store(brand: StoreBrand) -> Store {
        Store {
            brand,
            location: Coordinate::default(),
        }
    }

    #[test]
    fn routes_know_their_stores_even_at_one_location() {
        let stores = [
            store(StoreBrand::Paknsave),
            store(StoreBrand::Newworld),
            store(StoreBrand::Woolworths),
        ];
        let filters = StoreFilters::builder()
            .location(Coordinate::default())
            .range(Distance::from_kilometres_f64(25.0))
            .max_store_visits(3)
            .build();
        let visits: Vec<Vec<usize>> = all_possible_routes(&stores, &filters, &MileageOptions::Petrol)
            .into_iter()
            .map(|(visited, _)| visited)
            .collect();
        assert_eq!(
            visits,
            [vec![0], vec![1], vec![2], vec![0, 1], vec![0, 2], vec![1, 2], vec![0, 1, 2]]
        );
    }

    #[test]
    fn travel_cost_is_charged_per_kilometre() {
        let start = Coordinate::default();
        let path = RoutePlan {
            unordered_stops: [start.clone()].as_slice().into(),
            start_stop: start.clone(),
            end_stop: Coordinate::from_lat_long_f64(-41.289_848_883_193 + 0.009, 174.767_827_737_626_22),
            mileage: MileageOptions::Petrol,
        }
        .calculate();
        // about 1 km at $0.37/km is about 37 cents, not hundreds of dollars
        let cost = path.travel_cost.inner();
        assert!(cost > BigDecimal::from(30) / BigDecimal::from(100), "{cost}");
        assert!(cost < BigDecimal::from(45) / BigDecimal::from(100), "{cost}");
    }
}
