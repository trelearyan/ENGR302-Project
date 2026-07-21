use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::coordinate::Coordinate;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter, Serialize, Deserialize,
)]
pub enum StoreBrand {
    Paknsave,
    Newworld,
    Woolworths,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct Store {
    pub brand: StoreBrand,
    pub location: Coordinate,
}
