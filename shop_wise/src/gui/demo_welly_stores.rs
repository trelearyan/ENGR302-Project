//! Demo supermarket locations for the Wellington region.
//!
//! This is placeholder data standing in for a real store source. It is
//! embedded at compile time rather than read from disk, so it works
//! identically on the desktop build and in the browser.
//!
//! The coordinates are approximate. They are precise enough to tell stores
//! apart and to demonstrate the interface, but they have not been checked
//! against a map and should not be trusted for distance calculations.

use serde::Deserialize;
use util::store::StoreBrand;

/// One supermarket branch, as it appears in the demo data.
#[derive(Clone, Debug, Deserialize)]
pub struct DemoStore {
    pub id: u32,
    pub name: String,
    pub brand: StoreBrand,
    pub latitude: f64,
    pub longitude: f64,
}

const DEMO_STORES_JSON: &str = include_str!("demo_stores.json");

/// Every demo store, in the order they appear in the data file.
///
/// # Panics
///
/// Panics if the embedded data file is not valid JSON, which would be a
/// mistake in the file rather than anything a user could cause.
#[must_use]
pub fn all_demo_stores() -> Vec<DemoStore> {
    serde_json::from_str(DEMO_STORES_JSON).expect("demo_stores.json should be valid")
}

/// Demo stores belonging to one chain.
#[must_use]
pub fn demo_stores_for(brand: StoreBrand) -> Vec<DemoStore> {
    all_demo_stores()
        .into_iter()
        .filter(|store| store.brand == brand)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_data_file_parses() {
        assert!(!all_demo_stores().is_empty());
    }

    #[test]
    fn every_chain_has_at_least_one_store() {
        for brand in [
            StoreBrand::Paknsave,
            StoreBrand::Newworld,
            StoreBrand::Woolworths,
        ] {
            assert!(
                !demo_stores_for(brand).is_empty(),
                "no demo stores for {brand:?}"
            );
        }
    }

    #[test]
    fn store_ids_are_unique() {
        let stores = all_demo_stores();
        let mut ids: Vec<u32> = stores.iter().map(|store| store.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "demo store ids must be unique");
    }

    #[test]
    fn coordinates_are_within_the_wellington_region() {
        for store in all_demo_stores() {
            assert!(
                (-41.6..=-41.0).contains(&store.latitude),
                "{} has a latitude outside the region",
                store.name
            );
            assert!(
                (174.6..=175.2).contains(&store.longitude),
                "{} has a longitude outside the region",
                store.name
            );
        }
    }
}
