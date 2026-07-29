use std::{num::NonZeroU64, ops::Add};

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
#[display("{:.2}", inner)]
pub struct Cost {
    inner: BigDecimal,
}

impl Cost {
    // Per the Reserve Bank of New Zealand, 1-5c rounds down, 6-9c rounds up.
    // Source: https://web.archive.org/web/20111006085119/http://www.newcoins.govt.nz/1570749.html
    #[must_use]
    pub fn round(&self) -> Cost {
        Self::new(self.inner.with_precision_round(
            NonZeroU64::new(2).expect("This should be a compile time constant 2"),
            RoundingMode::HalfDown,
        ))
    }

    #[must_use]
    pub fn from_cents<NUMBER: Into<BigDecimal>>(cents: NUMBER) -> Self {
        Self::new(Into::<BigDecimal>::into(cents) / Into::<BigDecimal>::into(100))
    }

    #[must_use]
    pub fn inner(self) -> BigDecimal {
        self.inner
    }
}

impl<NUMBER: Into<BigDecimal>> From<NUMBER> for Cost {
    fn from(value: NUMBER) -> Self {
        Self::new(value.into())
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self::new(BigDecimal::zero())
    }
}
