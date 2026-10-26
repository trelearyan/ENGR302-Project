use std::str::FromStr;

use bigdecimal::BigDecimal;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::distance::Distance;

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
}

impl Default for Coordinate {
    fn default() -> Self {
        // VUW
        Self::from_lat_long_f64(-41.289_848_883_193, 174.767_827_737_626_22)
    }
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

    /// Creates a new [`Coordinate`].
    #[must_use]
    pub fn from_lat_long_f64(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude: BigDecimal::from_f64(latitude).unwrap(),
            longitude: BigDecimal::from_f64(longitude).unwrap(),
        }
    }

    /// Creates a new [`Coordinate`].
    #[must_use]
    pub fn from_lat_long_f32(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude: BigDecimal::from_f32(latitude).unwrap(),
            longitude: BigDecimal::from_f32(longitude).unwrap(),
        }
    }

    pub fn from_lat_long_str(
        latitude: &str,
        longitude: &str,
    ) -> Result<Self, <BigDecimal as FromStr>::Err> {
        Ok(Self {
            latitude: BigDecimal::from_str(latitude)?,
            longitude: BigDecimal::from_str(longitude)?,
        })
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
    pub fn within_range(&self, other: Self, range_metres: &Distance) -> bool {
        &self.distance_to(other) <= range_metres
    }

    #[must_use]
    pub fn wellington() -> Self {
        Coordinate::from_lat_long_str("-41.2866", "174.7756").unwrap()
    }
    #[must_use]
    pub fn auckland() -> Self {
        Coordinate::from_lat_long_str("-36.848461", "174.763336").unwrap()
    }
}
#[cfg(test)]
mod coordinate_tests {
    use crate::float_absolute_compare;
}
