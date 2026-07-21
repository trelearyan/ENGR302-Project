use bigdecimal::BigDecimal;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::distance::Distance;

#[derive(Default, Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
}

impl Coordinate {
    /// Creates a new [`Coordinate`].
    #[must_use]
    pub fn from_lat_long<T: Into<BigDecimal>>(latitude: T, longitude: T) -> Self {
        Self {
            latitude: latitude.into(),
            longitude: longitude.into(),
        }
    }

    #[must_use]
    /// Construct a
    pub fn distance_to(&self, other: Self) -> Distance {
        const ERROR_MESSAGE: &str =
            "This shouldnt be a weird float, if this fails, Coordinate is implemented wrong";
        Distance::from_metres(
            BigDecimal::from_f64(
                map_3d::distance(
                    (
                        self.latitude.to_f64().expect(ERROR_MESSAGE),
                        self.longitude.to_f64().expect(ERROR_MESSAGE),
                    ),
                    (
                        other.latitude.to_f64().expect(ERROR_MESSAGE),
                        other.longitude.to_f64().expect(ERROR_MESSAGE),
                    ),
                )
            ).expect("This shouldnt be a weird float, if this fails, Coordinate or map_3d is implemented wrong")
        )
    }

    #[must_use]
    pub fn within_range(&self, other: Self, range_metres: Distance) -> bool {
        self.distance_to(other) <= range_metres
    }
}

#[cfg(test)]
mod coordinate_tests {
    use crate::float_absolute_compare;
}
