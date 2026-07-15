use std::{any::Any, ops::Mul};

use crate::price_calculator::Supermarket;
use bigdecimal::BigDecimal;

pub struct Distance {
    pub metres: BigDecimal,
}

impl Distance {
    pub fn from_metres(metres: impl Into<BigDecimal>) -> Self {
        Self {
            metres: metres.into(),
        }
    }

    pub fn mileage(&self, cost_per_km: Cost) -> Cost {
        Cost::from_dollars(self.metres.clone() * 1000 * cost_per_km.dollars)
    }
}

pub struct Cost {
    pub dollars: BigDecimal,
}

impl Cost {
    pub fn from_dollars(dollars: impl Into<BigDecimal>) -> Self {
        Self {
            dollars: dollars.into(),
        }
    }
}

pub struct Stop {
    pub journey_distance: Distance,
    pub destination: Supermarket,
}

pub struct Route {
    pub stops: [Stop],
}

pub fn main_route_planner() {
    call_price_calculator()
}
