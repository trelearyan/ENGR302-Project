use std::num::NonZeroU64;

use bigdecimal::{BigDecimal, RoundingMode};
use derive_more::{
    Add, AddAssign, Constructor, Display, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
    SubAssign, Sum,
};

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Add,
    AddAssign,
    Constructor,
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
pub struct Cost {
    inner: BigDecimal,
}

impl Cost {
    // Per the Reserve Bank of New Zealand, 1-5c rounds down, 6-9c rounds up.
    // Source: https://web.archive.org/web/20111006085119/http://www.newcoins.govt.nz/1570749.html
    pub fn round(&self) -> Cost {
        Self::new(self.inner.with_precision_round(
            NonZeroU64::new(2).expect("This should be a compile time constant 2"),
            RoundingMode::HalfDown,
        ))
    }
}

impl<NUMBER: Into<BigDecimal>> From<NUMBER> for Cost {
    fn from(value: NUMBER) -> Self {
        Self::new(value.into())
    }
}
