use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use util::{
    coordinate::Coordinate,
    store::{Store, StoreBrand},
};

// TODO: this doc
/// simple explanation
///
/// detailed explanation
///
/// edge cases
///
/// example
/// ```
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StoreFilters {
    pub location: Coordinate,
    pub max_range_metres: Option<f32>,
    pub disallowed_brands: HashSet<StoreBrand>,
}

impl StoreFilters {
    // TODO: this doc
    /// simple explanation
    ///
    /// detailed explanation
    ///
    /// edge cases
    ///
    /// example
    /// ```
    /// ```
    pub fn new(
        location: Coordinate,
        max_range_metres: Option<f32>,
        disallowed_brands: HashSet<StoreBrand>,
    ) -> Self {
        Self {
            location,
            max_range_metres,
            disallowed_brands,
        }
    }

    // TODO: this doc
    /// simple explanation
    ///
    /// detailed explanation
    ///
    /// edge cases
    ///
    /// example
    /// ```
    /// ```
    pub fn builder() -> StoreFiltersBuilder {
        StoreFiltersBuilder::new()
    }
}

// TODO: this doc
/// simple explanation
///
/// detailed explanation
///
/// edge cases
///
/// example
/// ```
/// ```
impl Default for StoreFilters {
    fn default() -> Self {
        // See Self::builder().build() for default values
        Self::builder().build()
    }
}

// TODO: this doc
/// simple explanation
///
/// detailed explanation
///
/// edge cases
///
/// example
/// ```
/// ```
#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StoreFiltersBuilder {
    location: Option<Coordinate>,
    max_range_metres: Option<f32>,
    disallowed_brands: HashSet<StoreBrand>,
}

impl StoreFiltersBuilder {
    // TODO: this doc
    /// simple explanation
    ///
    /// detailed explanation
    ///
    /// edge cases
    ///
    /// example
    /// ```
    /// ```
    pub fn new() -> Self {
        Self {
            location: None,
            max_range_metres: None,
            disallowed_brands: HashSet::new(),
        }
    }

    // TODO: this doc
    /// Build the StoreFilters.
    ///
    /// detailed explanation
    ///
    /// edge cases
    ///
    /// example
    /// ```
    /// ```
    pub fn build(&mut self) -> StoreFilters {
        StoreFilters::new(
            // default: wellington
            self.location.unwrap_or(Coordinate::WELLINGTON),
            // default: None, or no range
            self.max_range_metres,
            // clone to allow reuse of builder
            self.disallowed_brands.clone(),
        )
    }

    // TODO: this doc
    /// Build the StoreFilters.
    ///
    /// detailed explanation
    ///
    /// edge cases
    ///
    /// example
    /// ```
    /// ```
    pub fn range(&mut self, range: f32) {
        self.max_range_metres = Some(range);
    }

    /// Set the centre point of the location filter.
    ///
    /// This function sets the location filter to the passed ``Coordinate``.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = StoreFilters::builder();
    /// filters.location(Coordinate::AUCKLAND);
    /// println!("{:?}", filters.build().location); // Coordinate { longitude: -36.84846, latitude: 174.76334 }
    /// ```
    pub fn location(&mut self, location: Coordinate) {
        self.location = Some(location);
    }

    /// Disallow a supermarket brand.
    ///
    /// This function adds brand to the set of disallowed brands.
    /// If brand is already in the set of disallowed brands, this function does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = StoreFilters::builder();
    /// filters.disallow_brand(StoreBrand::Woolworths);
    /// println!("{:?}", filters.build().allowed_brands); // {Woolworths}
    /// ```
    pub fn disallow_brand(&mut self, brand: StoreBrand) {
        self.disallowed_brands.insert(brand);
    }

    /// Disable multiple supermarket brands at once.
    ///
    /// This function runs ``[disallow_brand]`` for each ``StoreBrand`` in the passed slice.
    /// If brands contains multiple copies of a single brand, the extra copies will be ignored.
    /// If brands is empty, this function does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = StoreFilters::builder();
    /// filters.disallow_brands(&[StoreBrand::Paknsave, StoreBrand::Newworld]);
    /// println!("{:?}", filters.build().allowed_brands); // {Paknsave, NewWorld}
    /// ```
    pub fn disallow_brands(&mut self, brands: &[StoreBrand]) {
        for brand in brands {
            self.disallowed_brands.insert(*brand);
        }
    }

    /// Enable all supermarket brands.
    ///
    /// This function clears the set of disallowed brands, meaning no stores will be filtered out based on their brand.
    /// If the set of disallowed brands is already empty, this function does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = StoreFilters::builder();
    /// filters.disallow_brand(StoreBrand::Paknsave);
    /// println!("{:?}", filters.build().allowed_brands); // {Paknsave}
    /// filters.all_brands();
    /// println!("{:?}", filters.build().allowed_brands); // {}
    /// ```
    pub fn all_brands(&mut self) {
        self.disallowed_brands.clear();
    }
}

// TODO: this doc
/// simple explanation
///
/// detailed explanation
///
/// edge cases
///
/// example
/// ```
/// ```
pub fn filter_stores(stores: &[Store], filters: StoreFilters) -> Vec<Store> {
    // stores
    //     .iter()
    //     .copied()
    //     .filter(|store| {
    //         if filters.disallowed_brands.contains(&store.brand) {
    //             eprintln!(
    //                 "store {store:?} is in banlist {:?}",
    //                 filters.disallowed_brands
    //             );
    //             false
    //         } else {
    //             true
    //         }
    //     })
    //     .filter(|store| match filters.max_range_metres {
    //         Some(range) => {
    //             if filters.location.within_range(store.location, range) {
    //                 eprintln!(
    //                     "store {store:?} is out of range of {:?} max range is {range:?}",
    //                     filters.location
    //                 );
    //                 false
    //             } else {
    //                 eprintln!(
    //                     "store {store:?} is in range of {:?} max range is {range:?}",
    //                     filters.location
    //                 );
    //                 true
    //             }
    //         }
    //         None => true,
    //     })
    //     .collect::<Vec<_>>()

    stores
        .iter()
        .copied()
        .filter(|store| !filters.disallowed_brands.contains(&store.brand))
        .filter(|_store| true)
        //     match filters.max_range_metres {
        //     // TODO: fix this
        //     Some(range) => filters.location.within_range(store.location, range),
        //     None => true,
        // })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use serde_json::Deserializer;

    use super::*;

    const UNIVERSITY: Coordinate = Coordinate::with_decimal_degrees(-41.290_38, 174.767_94);
    const TE_PAPA: Coordinate = Coordinate::with_decimal_degrees(-41.290_474, 174.781_02);

    const CABLE_CAR_WOOLIES: Store = Store {
        brand: StoreBrand::Woolworths,
        location: Coordinate::with_decimal_degrees(-41.284_46, 174.775_01),
    };
    const NEWTOWN_WOOLIES: Store = Store {
        brand: StoreBrand::Woolworths,
        location: Coordinate::with_decimal_degrees(-41.307_335, 174.777_42),
    };
    const NEWWORLD_METRO: Store = Store {
        brand: StoreBrand::Newworld,
        location: Coordinate::with_decimal_degrees(-41.287_785, 174.775_25),
    };
    const NEWWORLD_SHAFFERS: Store = Store {
        brand: StoreBrand::Newworld,
        location: Coordinate::with_decimal_degrees(-41.292_35, 174.784_29),
    };
    const PAKNSAVE_KILBIRNE: Store = Store {
        brand: StoreBrand::Paknsave,
        location: Coordinate::with_decimal_degrees(-41.318_737, 174.796_69),
    };
    const NEWWORLD_NEWTOWN: Store = Store {
        brand: StoreBrand::Newworld,
        location: Coordinate::with_decimal_degrees(-41.307_47, 174.777_48),
    };

    const EXAMPLE_DATABASE: &[Store] = &[
        CABLE_CAR_WOOLIES,
        NEWTOWN_WOOLIES,
        NEWWORLD_METRO,
        NEWWORLD_SHAFFERS,
        PAKNSAVE_KILBIRNE,
        NEWWORLD_NEWTOWN,
    ];

    #[test]
    fn test_filter_stores() {
        let mut builder = StoreFilters::builder();

        builder.location(UNIVERSITY);
        builder.range(5.);
        builder.disallow_brands(&[StoreBrand::Paknsave, StoreBrand::Woolworths]);

        println!(" -- FILTERS -- ");
        println!(
            "{}",
            serde_json::to_string_pretty(&builder.build()).unwrap()
        );
        println!(" --  -- ");
        println!(" --  -- ");

        println!(" -- STORES -- ");
        println!(
            "{}",
            serde_json::to_string_pretty(&EXAMPLE_DATABASE).unwrap()
        );
        println!(" --  -- ");
        println!(" --  -- ");

        println!(" -- OUTPUT -- ");
        println!(
            "{}",
            serde_json::to_string_pretty(&filter_stores(EXAMPLE_DATABASE, builder.build()))
                .unwrap()
        );
        println!(" --  -- ");
        println!(" --  -- ");
    }

    // #[test]
    // pub fn demonstrate() {
    //     let filters =
    // }
}
