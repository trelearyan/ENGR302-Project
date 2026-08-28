use std::ops::Mul;

use bigdecimal::BigDecimal;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};
use eframe::emath::Numeric;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

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
    base_value_metres: BigDecimal,
}

impl Distance {
    pub fn from_metres_f64(metres: f64) -> Self {
        Self::from_metres(BigDecimal::from_f64(metres).unwrap())
    }

    pub fn from_kilometres_f64(metres: f64) -> Self {
        Self::from_metres(BigDecimal::from_f64(metres * 1000.).unwrap())
    }

    /// if metres is negative, it becomes positive
    pub fn from_metres<T: Into<BigDecimal>>(metres: T) -> Self {
        Self {
            base_value_metres: metres.into().abs(),
        }
    }

    pub fn metres(&self) -> f64 {
        self.base_value_metres.clone().to_f64().unwrap()
    }

    pub fn kilometres(&self) -> f64 {
        (self.base_value_metres.clone() / BigDecimal::from(1000))
            .to_f64()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use eframe::wgpu::wgt::strict_assert_eq;
    use num_traits::Float;

    use super::*;
    use std::{fmt::Debug, str::FromStr};

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
}
