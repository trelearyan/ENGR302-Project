use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::coordinate::Coordinate;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter, Serialize, Deserialize,
)]
pub enum StoreBrand {
    #[serde(rename = "Pak'nSave")]
    Paknsave,

    #[serde(rename = "New World")]
    Newworld,

    #[serde(rename = "Woolworths")]
    Woolworths,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Store {
    pub brand: StoreBrand,
    pub location: Coordinate,
}
