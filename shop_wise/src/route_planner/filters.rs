use std::collections::HashSet;
use strum::IntoEnumIterator;
use util::{
    coordinate::Coordinate,
    store::{Store, StoreBrand},
};
#[derive(Debug, PartialEq, Clone)]
pub struct StoreFilters {
    location: Coordinate,
    max_range_metres: Option<f32>,
    allowed_brands: HashSet<StoreBrand>,
}

impl Default for StoreFilters {
    fn default() -> Self {
        // See Self::builder().build() for default values
        Self::builder().build()
    }
}

impl StoreFilters {
    pub fn new(
        location: Coordinate,
        max_range_metres: Option<f32>,
        allowed_brands: HashSet<StoreBrand>,
    ) -> Self {
        Self {
            location,
            max_range_metres,
            allowed_brands,
        }
    }

    pub fn builder() -> StoreFiltersBuilder {
        StoreFiltersBuilder::new()
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct StoreFiltersBuilder {
    location: Option<Coordinate>,
    max_range_metres: Option<f32>,
    allowed_brands: Option<HashSet<StoreBrand>>,
}

impl StoreFiltersBuilder {
    pub fn new() -> Self {
        Self {
            location: None,
            max_range_metres: None,
            allowed_brands: None,
        }
    }

    pub fn build(&mut self) -> StoreFilters {
        StoreFilters::new(
            // default: wellington
            self.location.unwrap_or(Coordinate::WELLINGTON),
            // default: None, or no range
            self.max_range_metres,
            // default: all brands allowed
            self.allowed_brands
                .clone() // unavoidable, to allow for using builder multiple times
                .unwrap_or(StoreBrand::iter().collect::<HashSet<_>>()),
        )
    }

    pub fn location(&mut self, location: Coordinate) {
        self.location = Some(location);
    }

    pub fn add_brand(&mut self, brand: StoreBrand) {
        match self.allowed_brands {
            Some(ref mut set) => {
                set.insert(brand);
            }
            None => {
                self.allowed_brands = Some(HashSet::from([brand]));
            }
        }
    }

    pub fn set_brands(&mut self, brands: &[StoreBrand]) {
        // just plain enum variants so copy is fine
        self.allowed_brands = Some(brands.iter().collect::<HashSet<_>>());
    }

    pub fn all_brands(&mut self) {
        self.allowed_brands = Some(HashSet::from_iter(StoreBrand::iter()));
    }
}

pub fn filter_stores(stores: &[Store], filters: StoreFilters) -> Vec<Store> {
    stores
        .iter()
        .copied()
        .filter(|store| filters.allowed_brands.contains(&store.brand))
        .filter(|store| match filters.max_range_metres {
            Some(range) => filters.location.within_range(store.location, range),
            None => true,
        })
        .collect::<Vec<_>>()
}
