use std::{ops::Div, time::Duration};

use bigdecimal::BigDecimal;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::speed::Speed;

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Sum,
)]

pub struct Distance {
    pub(crate) base_value_metres: BigDecimal,
}

impl Distance {
    #[must_use]
    pub fn from_metres_f64(metres: f64) -> Self {
        Self::from_metres(BigDecimal::from_f64(metres).unwrap())
    }

    #[must_use]
    pub fn from_kilometres_f64(metres: f64) -> Self {
        Self::from_metres(BigDecimal::from_f64(metres * 1000.).unwrap())
    }

    /// if metres is negative, it becomes positive
    pub fn from_metres<T: Into<BigDecimal>>(metres: T) -> Self {
        Self {
            base_value_metres: metres.into().abs(),
        }
    }

    #[must_use]
    pub fn metres(&self) -> f64 {
        self.base_value_metres.clone().to_f64().unwrap()
    }

    #[must_use]
    pub fn kilometres(&self) -> f64 {
        (self.base_value_metres.clone() / BigDecimal::from(1000))
            .to_f64()
            .unwrap()
    }

    #[must_use]
    pub fn inner(&self) -> BigDecimal {
        self.base_value_metres.clone()
    }
}

impl Div<Speed> for Distance {
    type Output = Duration;

    fn div(self, rhs: Speed) -> Self::Output {
        Duration::from_secs((self.base_value_metres / rhs.inner()).to_u64().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_within_tolerance(x: f64, y: f64) {
        const ERROR_EPSILON: f64 = 0.001;

        assert!((x - y).abs() < ERROR_EPSILON);
    }

    #[test]
    fn kilometers_conversion() {
        fn test(number: f64) {
            let d38_1 = Distance::from_metres_f64(number * 1000.);
            let d38_2 = Distance::from_kilometres_f64(number);

            assert_eq!(d38_2, d38_1);
            assert_within_tolerance(d38_2.metres(), d38_1.metres());
            assert_within_tolerance(d38_2.kilometres(), d38_1.kilometres());

            assert_within_tolerance(d38_2.metres(), number * 1000.);
            assert_within_tolerance(d38_2.kilometres(), number);

            assert_within_tolerance(d38_1.metres(), number * 1000.);
            assert_within_tolerance(d38_1.kilometres(), number);
        }

        test(38.);
        test(0.);
        test(0.001);
        test(899_999.);
    }

    #[test]
    fn test_div_by_speed() {
        assert_eq!(Duration::from_secs(100), Distance::from_metres(1000) / Speed::from_metres_per_second(10));
        assert_eq!(Duration::from_secs(10), Distance::from_metres(1000) / Speed::from_metres_per_second(100));
        assert_eq!(Duration::from_secs(10), Distance::from_metres(100) / Speed::from_metres_per_second(10));
        assert_eq!(Duration::from_secs(1000), Distance::from_metres(1000) / Speed::from_metres_per_second(1));

        assert_eq!(Duration::from_secs(100), Distance::from_metres(1000) / Speed::from_kilometres_per_hour(36));
        assert_eq!(Duration::from_secs(1), Distance::from_metres(1000) / Speed::from_kilometres_per_hour(3600));
        assert_eq!(Duration::from_secs(1), Distance::from_metres(100) / Speed::from_kilometres_per_hour(360));
        assert_eq!(Duration::from_secs(1000), Distance::from_metres(10000) / Speed::from_kilometres_per_hour(36));

        assert_eq!(Duration::from_secs(100), Distance::from_kilometres_f64(1.0) / Speed::from_metres_per_second(10));
        assert_eq!(Duration::from_secs(10), Distance::from_kilometres_f64(1.0) / Speed::from_metres_per_second(100));
        assert_eq!(Duration::from_secs(10), Distance::from_kilometres_f64(0.1) / Speed::from_metres_per_second(10));
        assert_eq!(Duration::from_secs(1000), Distance::from_kilometres_f64(1.0) / Speed::from_metres_per_second(1));

        assert_eq!(Duration::from_mins(1), Distance::from_metres(5400) / Speed::from_metres_per_second(90));
        assert_eq!(Duration::from_mins(10), Distance::from_metres(54000) / Speed::from_metres_per_second(90));
        assert_eq!(Duration::from_mins(10), Distance::from_metres(5400) / Speed::from_metres_per_second(9));
        assert_eq!(Duration::from_mins(1), Distance::from_metres(54000) / Speed::from_metres_per_second(900));
    }
}
