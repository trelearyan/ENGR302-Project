use nalgebra::Vector3;

#[derive(Default, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Coordinate {
    longitude: f32,
    latitude: f32,
}

impl Coordinate {
    pub const fn new(longitude: f32, latitude: f32) -> Self {
        Self {
            longitude,
            latitude,
        }
    }

    pub fn geodetic_normal(&self) -> Vector3<f32> {
        // https://en.wikipedia.org/wiki/N-vector#Converting_latitude/longitude_to_n-vector
        Vector3::new(
            self.latitude.cos() * self.longitude.cos(),
            self.latitude.sin() * self.longitude.sin(),
            self.latitude.sin(),
        )
    }

    pub fn distance_between(&self, other: Self) -> f32 {
        const EARTH_RADIUS_METRES: f32 = 6_371_000.;
        // https://en.wikipedia.org/wiki/N-vector#Example_1:_Great_circle_distance
        let a = self.geodetic_normal();
        let b = other.geodetic_normal();

        let theta = (a.cross(&b).magnitude() / a.dot(&b)).atan();

        let result = theta * EARTH_RADIUS_METRES;

        result
    }

    pub fn within_range(&self, other: Self, range_metres: f32) -> bool {
        self.distance_between(other) <= range_metres
    }

    // Order is degree, min, second. If mins or seconds is negative, that works to subtract from degrees or mins respectively
    pub const fn with_degrees_minutes_seconds(
        longitude: (f32, f32, f32),
        latitude: (f32, f32, f32),
    ) -> Self {
        Self::new(
            longitude.2 + longitude.1 * 60. + longitude.0 * 60. * 60.,
            latitude.2 + latitude.1 * 60. + latitude.0 * 60. * 60.,
        )
    }

    pub const fn with_decimal_degrees(longitude: f32, latitude: f32) -> Self {
        Self::new(longitude, latitude)
    }

    pub const WELLINGTON: Self = Coordinate::with_decimal_degrees(-41.294_769, 174.771_973);
    pub const AUCKLAND: Self = Coordinate::with_decimal_degrees(-36.848_461, 174.763_336);
    pub const CHRISTCHURCH: Self = Coordinate::with_decimal_degrees(-43.525_650, 172.639_847);
}
#[cfg(test)]
mod coordinate_tests {
    use super::Coordinate;
    #[test]
    fn test_wellington_conversion() {
        let wellington_dd = Coordinate::with_decimal_degrees(-41.28664, 174.77557);
        // -41°17'11.90" S 174°46'32.05"
        let wellington_dms =
            Coordinate::with_degrees_minutes_seconds((-41., 17., 11.), (174., 46., 32.));
        assert_eq!(wellington_dms, wellington_dd);
    }
}
