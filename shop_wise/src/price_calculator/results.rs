use std::sync::Arc;

use crate::price_calculator::{Calculation, CalculationTotal};
use bigdecimal::{BigDecimal, Zero};
use routx::Graph;
use serde::{Deserialize, Serialize};
use util::coordinate::Coordinate;
use util::cost::Cost;
use util::distance::Distance;
use util::store::StoreBrand;

//TODO: remove once all have values
pub const UNAVAILABLE: &str = "not available";
#[must_use]
pub fn format_money(cost: &Cost) -> String {
    format!("${cost}")
}

#[must_use]
pub fn format_delta(cost: &Cost, base: &Cost) -> Option<String> {
    let (sign, difference) = if cost < base {
        ('-', base.clone() - cost.clone())
    } else {
        ('+', cost.clone() - base.clone())
    };
    let amount = format_money(&difference);
    (amount != "$0.00").then(|| format!("{sign}{amount}"))
}

#[must_use]
pub fn format_distance(distance: &Distance) -> String {
    format!("{:.1} km", distance.kilometres())
}

#[must_use]
pub fn format_duration(minutes: f32) -> String {
    let total = minutes.round().max(0.0) as u32;
    if total < 60 {
        format!("{total:.1} min")
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
            ScenarioKind::Best => "Best Value",
            ScenarioKind::Cheapest => "Cheapest",
            ScenarioKind::Fastest => "Fastest",
        }
    }
    #[must_use]
    pub fn icon(self) -> &'static str {
        match self {
            ScenarioKind::Cheapest => "🏷",
            ScenarioKind::Fastest => "⚡",
            ScenarioKind::Best => "☆",
        }
    }

    #[must_use]
    pub fn blurb(self) -> &'static str {
        match self {
            ScenarioKind::Best => "Balances money saved against time spent",
            ScenarioKind::Cheapest => "Lowest total cost",
            ScenarioKind::Fastest => "Shortest trip",
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
    pub quantity: String,
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
    pub ordered_stops: Arc<[Coordinate]>,
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
            .map(|i| i.multiplier)
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

    fn from_calculation(kind: ScenarioKind, calc: &Calculation, graph: &Graph) -> Self {
        let store_plans = &calc.shopping_plan;
        //store_ids.sort();

        let stops = store_plans
            .iter()
            .filter_map(|infos| {
                let brand = infos.store.brand;

                let items = infos
                    .items
                    .iter()
                    .map(|info| ItemLine {
                        name: info.name.clone(),
                        quantity: info.multiplier.to_string()
                            + "x "
                            + &info.quantity.to_string()
                            + info.unit.to_str(),
                        unit_price: info.price.clone(),
                        on_special: false,
                        needs_loyalty_card: false,
                        multiplier: info.multiplier,
                    })
                    .collect();

                Some(StoreStop {
                    store_name: None,
                    chain: brand,
                    address: None,
                    items,
                })
            })
            .collect();

        log::debug!(
            "travel distance is {:?}",
            calc.source.exact_travel_distance(graph)
        );

        log::debug!(
            "travel time is {:?}s or {:?}m",
            calc.source
                .exact_travel_time(graph, &calc.mileage)
                .as_secs_f32(),
            calc.source
                .exact_travel_time(graph, &calc.mileage)
                .as_secs_f32()
                / 60f32,
        );

        Self {
            kind,
            stops,
            travel_cost: Some(calc.source.exact_travel_cost(graph, &calc.mileage)),
            distance_km: Some(calc.source.exact_travel_distance(graph)),
            duration_min: Some(
                calc.source
                    .exact_travel_time(graph, &calc.mileage)
                    .as_secs_f32()
                    / 60f32,
            ),
            ordered_stops: calc.source.expensive_calculated_route.wait().clone(),
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

    /// One line weighing `kind` against the fastest plan (or the cheapest, for the fastest
    /// plan itself). `None` when there is no real money-for-time trade-off.
    #[must_use]
    pub fn trade_off(&self, kind: ScenarioKind) -> Option<String> {
        let chosen = self.scenario(kind);
        if kind == ScenarioKind::Fastest {
            let extra = chosen.total_cost() - self.cheapest.total_cost();
            let faster = self.cheapest.duration_min? - chosen.duration_min?;
            (extra > Cost::default() && faster >= 1.0).then(|| {
                format!(
                    "{} faster than the cheapest plan, for an extra {}.",
                    format_duration(faster),
                    format_money(&extra)
                )
            })
        } else {
            let saving = self.fastest.total_cost() - chosen.total_cost();
            let slower = chosen.duration_min? - self.fastest.duration_min?;
            (saving > Cost::default() && slower >= 1.0).then(|| {
                format!(
                    "{} cheaper than the fastest plan, for an extra {}.",
                    format_money(&saving),
                    format_duration(slower)
                )
            })
        }
    }

    #[must_use]
    pub fn from_calculation(calc: &CalculationTotal, origin_label: String, graph: &Graph) -> Self {
        Self {
            origin_label,
            best: Scenario::from_calculation(ScenarioKind::Best, &calc.best, graph),
            cheapest: Scenario::from_calculation(ScenarioKind::Cheapest, &calc.cheapest, graph),
            fastest: Scenario::from_calculation(ScenarioKind::Fastest, &calc.fastest, graph),
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
    fn delta_is_signed() {
        let base = Cost::from_cents(6140);
        assert_eq!(
            format_delta(&Cost::from_cents(6890), &base).as_deref(),
            Some("+$7.50")
        );
        assert_eq!(
            format_delta(&Cost::from_cents(5960), &base).as_deref(),
            Some("-$1.80")
        );
        assert_eq!(format_delta(&base, &base), None);
    }

    #[test]
    fn trade_off_compares_money_and_time() {
        let plan = |kind, cents, minutes| Scenario {
            kind,
            stops: Vec::new(),
            travel_cost: Some(Cost::from_cents(cents)),
            distance_km: None,
            duration_min: Some(minutes),
            ordered_stops: Arc::new([]),
        };
        let mut results = Results {
            origin_label: String::new(),
            best: plan(ScenarioKind::Best, 6320, 22.0),
            cheapest: plan(ScenarioKind::Cheapest, 6140, 38.0),
            fastest: plan(ScenarioKind::Fastest, 6890, 14.0),
            unresolved: Vec::new(),
        };
        assert_eq!(
            results.trade_off(ScenarioKind::Cheapest).as_deref(),
            Some("$7.50 cheaper than the fastest plan, for an extra 24 min.")
        );
        assert_eq!(
            results.trade_off(ScenarioKind::Fastest).as_deref(),
            Some("24 min faster than the cheapest plan, for an extra $7.50.")
        );

        // same plan everywhere: nothing to weigh up
        results.fastest = plan(ScenarioKind::Fastest, 6140, 38.0);
        assert_eq!(results.trade_off(ScenarioKind::Cheapest), None);
        assert_eq!(results.trade_off(ScenarioKind::Fastest), None);
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
                multiplier: 1,
                quantity: "1x 2L".to_owned(),
                unit_price: Cost::from_cents(898),
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
            ordered_stops: Arc::new([]),
        };
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 1);
    }

    #[test]
    fn total_is_groceries_plus_travel2() {
        let stop = StoreStop {
            store_name: Some("Test".to_owned()),
            chain: StoreBrand::Paknsave,
            address: Some(String::new()),
            items: vec![ItemLine {
                name: "Milk 1L".to_owned(),
                multiplier: 2,
                quantity: "2x 1L".to_owned(),
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
            ordered_stops: Arc::new([]),
        };
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 2);
    }
}
