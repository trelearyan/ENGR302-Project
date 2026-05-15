use strum_macros::EnumIter;

use crate::coordinate::Coordinate;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter)]
pub enum StoreBrand {
    Paknsave,
    Newworld,
    Woolworths,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Store {
    pub brand: StoreBrand,
    pub location: Coordinate,
}
