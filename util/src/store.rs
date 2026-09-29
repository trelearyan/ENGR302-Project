use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use crate::coordinate::Coordinate;

/// A supermarket chain whose prices can be compared.
///
/// When serialized, each chain uses its brand name as written in the
/// `supermarket` field of the `data/*_locations.json` files, so those files
/// can be read straight into this type.
///
/// # Examples
///
/// ```
/// use util::store::StoreBrand;
///
/// # fn main() -> Result<(), serde_json::Error> {
/// let brand: StoreBrand = serde_json::from_str(r#""Pak'nSave""#)?;
/// assert_eq!(brand, StoreBrand::Paknsave);
///
/// assert_eq!(serde_json::to_string(&StoreBrand::Newworld)?, r#""New World""#);
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter, Serialize, Deserialize,
)]
pub enum StoreBrand {
    /// Pak'nSave: Serialised as `"Pak'nSave"`.
    #[serde(rename = "Pak'nSave")]
    Paknsave,

    /// New World: Serialised as `"New World"`.
    #[serde(rename = "New World")]
    Newworld,

    /// Woolworths: Serialised as `"Woolworths"`.
    #[serde(rename = "Woolworths")]
    Woolworths,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Store {
    pub brand: StoreBrand,
    pub location: Coordinate,
}
