use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};
use util::cost::Cost;
use util::distance::Distance;
use util::store::StoreBrand;

// TODO(FR-09):
// use std::collections::HashMap;
//use crate::price_calculator::StoreId;
use crate::price_calculator::{Calculation, CalculationTotal};

/// for data that has no output yet
pub const UNAVAILABLE: &str = "not available";
#[must_use]
pub fn format_money(cost: &Cost) -> String {
    format!("${cost}")
}

#[must_use]
pub fn format_distance(distance: &Distance) -> String {
    format!("{:.1} km", distance.kilometres())
}

#[must_use]
pub fn format_duration(minutes: f32) -> String {
    let total = minutes.round().max(0.0) as u32;
    if total < 60 {
        format!("{total} min")
    } else {
        format!("{} h {:02} min", total / 60, total % 60)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScenarioKind {
    Best,
    Cheapest,
    Fastest,
}

impl ScenarioKind {
    pub const ALL: [ScenarioKind; 3] = [
        ScenarioKind::Best,
        ScenarioKind::Cheapest,
        ScenarioKind::Fastest,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ScenarioKind::Best => "Best value",
            ScenarioKind::Cheapest => "Cheapest",
            ScenarioKind::Fastest => "Fastest",
        }
    }

    #[must_use]
    pub fn blurb(self) -> &'static str {
        match self {
            ScenarioKind::Best => "Balances money saved against time spent",
            ScenarioKind::Cheapest => "Lowest groceries plus petrol",
            ScenarioKind::Fastest => "Shortest trip that still gets everything",
        }
    }
}

#[must_use]
pub fn brand_label(brand: StoreBrand) -> &'static str {
    match brand {
        StoreBrand::Paknsave => "Pak'nSave",
        StoreBrand::Newworld => "New World",
        StoreBrand::Woolworths => "Woolworths",
    }
}

#[derive(Clone, Debug)]
pub struct ItemLine {
    pub name: String,
    pub multiplier: u32,
    pub quantity: u32,
    pub unit_price: Cost,
    pub on_special: bool,
    pub needs_loyalty_card: bool,
}

impl ItemLine {
    #[must_use]
    pub fn line_total(&self) -> Cost {
        self.unit_price.clone() * BigDecimal::from(self.multiplier)
    }
}

#[derive(Clone, Debug)]
pub struct StoreStop {
    pub store_name: Option<String>,
    pub chain: StoreBrand,
    pub address: Option<String>,
    pub items: Vec<ItemLine>,
}

impl StoreStop {
    pub fn subtotal(&self) -> Cost {
        self.items.iter().map(ItemLine::line_total).sum()
    }
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub kind: ScenarioKind,
    pub stops: Vec<StoreStop>,
    //missing
    pub travel_cost: Option<Cost>,
    pub distance_km: Option<Distance>,
    pub duration_min: Option<f32>,
}

impl Scenario {
    pub fn grocery_cost(&self) -> Cost {
        self.stops.iter().map(StoreStop::subtotal).sum()
    }

    #[must_use]
    pub fn total_cost(&self) -> Cost {
        match &self.travel_cost {
            Some(travel) => self.grocery_cost() + travel.clone(),
            None => self.grocery_cost(),
        }
    }
    #[must_use]
    pub fn item_count(&self) -> u32 {
        self.stops
            .iter()
            .flat_map(|s| s.items.iter())
            .map(|i| i.quantity)
            .sum()
    }

    #[must_use]
    pub fn store_summary(&self) -> String {
        if self.stops.is_empty() {
            return "No store found".to_owned();
        }
        self.stops
            .iter()
            .map(|s| {
                s.store_name
                    .clone()
                    .unwrap_or_else(|| brand_label(s.chain).to_owned())
            })
            .collect::<Vec<_>>()
            .join(" + ")
    }
    // TODO(FR-09):when Results::from_calculation works

    fn from_calculation(kind: ScenarioKind, calc: &Calculation) -> Self {
        let store_plans = &calc.shopping_plan;
        //store_ids.sort();

        let stops = store_plans
            .iter()
            .filter_map(|infos| {
                // TODO: shopping_plan store name
                // TODO: fix match_sid_to_brand and calculate mapping
                let brand = infos.store.brand;

                let items = infos
                    .items
                    .iter()
                    .map(|info| ItemLine {
                        name: info.name.clone(),
                        quantity: info.quantity,
                        unit_price: info.price.clone(),
                        // TODO: loyalty pricing
                        on_special: false,
                        needs_loyalty_card: false,
                        multiplier: info.multiplier,
                    })
                    .collect();

                Some(StoreStop {
                    store_name: None,
                    chain: brand,
                    // TODO: no address in output.
                    address: None,
                    items,
                })
            })
            .collect();

        Self {
            kind,
            stops,
            // TODO: calc output has no travel, distance and driving time
            travel_cost: Some(calc.total_travel_cost.clone()),
            distance_km: Some(calc.total_dist.clone()),
            duration_min: Some((calc.total_time.as_secs() / 60) as f32),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnresolvedReason {
    NotRecognised,
    NotStockedNearby,
}

impl UnresolvedReason {
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            UnresolvedReason::NotRecognised => "not recognised, check the spelling",
            UnresolvedReason::NotStockedNearby => "not stocked by any store in range",
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnresolvedItem {
    pub raw_text: String,
    pub reason: UnresolvedReason,
}

#[derive(Clone, Debug)]
pub struct Results {
    pub origin_label: String,
    pub best: Scenario,
    pub cheapest: Scenario,
    pub fastest: Scenario,
    pub unresolved: Vec<UnresolvedItem>,
}

impl Results {
    #[must_use]
    pub fn scenario(&self, kind: ScenarioKind) -> &Scenario {
        match kind {
            ScenarioKind::Best => &self.best,
            ScenarioKind::Cheapest => &self.cheapest,
            ScenarioKind::Fastest => &self.fastest,
        }
    }

    #[must_use]
    pub fn headline_saving(&self) -> Option<Cost> {
        let dearest = ScenarioKind::ALL
            .iter()
            .map(|k| self.scenario(*k).total_cost())
            .max()?;
        let saving = dearest - self.cheapest.total_cost();
        (saving.clone().inner() > BigDecimal::zero()).then_some(saving)
    }

    // TODO(FR-09): once fields are pub

    #[must_use]
    pub fn from_calculation(calc: &CalculationTotal, origin_label: String) -> Self {
        Self {
            origin_label,
            best: Scenario::from_calculation(ScenarioKind::Best, &calc.best),
            cheapest: Scenario::from_calculation(ScenarioKind::Cheapest, &calc.cheapest),
            fastest: Scenario::from_calculation(ScenarioKind::Fastest, &calc.fastest),
            // TODO: resolve() failures
            unresolved: Vec::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub enum ResultsState {
    Idle,
    Loading,
    Ready(Box<Results>),
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_formats_with_two_decimals() {
        assert_eq!(format_money(&Cost::from_cents(0)), "$0.00");
        assert_eq!(format_money(&Cost::from_cents(5)), "$0.05");
        assert_eq!(format_money(&Cost::from_cents(1234)), "$12.34");
    }

    #[test]
    fn duration_rolls_over_to_hours() {
        assert_eq!(format_duration(19.4), "19 min");
        assert_eq!(format_duration(65.0), "1 h 05 min");
    }

    #[test]
    fn total_is_groceries_plus_travel() {
        let stop = StoreStop {
            store_name: Some("Test".to_owned()),
            chain: StoreBrand::Paknsave,
            address: Some(String::new()),
            items: vec![ItemLine {
                name: "Milk 2L".to_owned(),
                quantity: 2,
                unit_price: Cost::from_cents(449),
                on_special: false,
                needs_loyalty_card: false,
            }],
        };
        let scenario = Scenario {
            kind: ScenarioKind::Cheapest,
            stops: vec![stop],
            travel_cost: Some(Cost::from_cents(380)),
            distance_km: Some(Distance::from_kilometres_f64(8.6)),
            duration_min: Some(19.0),
        };
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 2);
    }
}
