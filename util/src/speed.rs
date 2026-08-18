use std::{num::NonZeroU64, ops::Add, str::FromStr};

use bigdecimal::{BigDecimal, BigDecimalRef, RoundingMode, Zero};
use derive_more::{
    Add, AddAssign, Constructor, Display, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
    SubAssign, Sum,
};

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Add,
    AddAssign,
    Display,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Neg,
    Rem,
    RemAssign,
    Sub,
    SubAssign,
    Sum,
)]
#[display("{:.2}", inner)]
pub struct Speed {
    inner: BigDecimal,
}

impl Speed {
    fn new(mps: BigDecimal) -> Self {
        Self { inner: mps.abs() }
    }

    #[must_use]
    pub fn from_metres_per_second<NUMBER: Into<BigDecimal>>(mps: NUMBER) -> Self {
        Self::new(mps.into())
    }

    #[must_use]
    pub fn from_kilometres_per_hour<NUMBER: Into<BigDecimal>>(kph: NUMBER) -> Self {
        Self::new(
            kph.into() * BigDecimal::from_str("3.6").expect("3.6 should be a valid BigDecimal"),
        )
    }

    #[must_use]
    pub fn inner(self) -> BigDecimal {
        self.inner
    }
}

impl<NUMBER: Into<BigDecimal>> From<NUMBER> for Speed {
    fn from(value: NUMBER) -> Self {
        Self::new(value.into())
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(BigDecimal::zero())
    }
}
