use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use util::cost::Cost;
use util::distance::Distance;
use util::store::StoreBrand;

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

/// Formats `value` with `format`, or gives [`UNAVAILABLE`] when there is no value.
#[must_use]
pub fn format_or_unavailable<T>(value: Option<T>, format: impl FnOnce(T) -> String) -> String {
    value.map_or_else(|| UNAVAILABLE.to_owned(), format)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScenarioKind {
    #[default]
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
    /// The store's own name, or its chain's name when it has none.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.store_name
            .as_deref()
            .unwrap_or(brand_label(self.chain))
    }

    #[must_use]
    pub fn subtotal(&self) -> Cost {
        self.items.iter().map(ItemLine::line_total).sum()
    }
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub kind: ScenarioKind,
    pub stops: Vec<StoreStop>,
    // `None` is shown as `UNAVAILABLE`.
    pub travel_cost: Option<Cost>,
    pub distance_km: Option<Distance>,
    pub duration_min: Option<f32>,
}

impl Scenario {
    #[must_use]
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
            .map(StoreStop::display_name)
            .collect::<Vec<_>>()
            .join(" + ")
    }

    fn from_calculation(kind: ScenarioKind, calc: &Calculation) -> Self {
        let stops = calc
            .shopping_plan
            .iter()
            .map(|plan| StoreStop {
                // TODO: shopping_plan store name
                store_name: None,
                chain: plan.store.brand,
                // TODO: no address in output.
                address: None,
                items: plan
                    .items
                    .iter()
                    .map(|info| ItemLine {
                        name: info.name.clone(),
                        multiplier: info.multiplier,
                        quantity: format!(
                            "{}x {}{}",
                            info.multiplier,
                            info.quantity,
                            info.unit.to_str()
                        ),
                        unit_price: info.price.clone(),
                        // TODO: loyalty pricing
                        on_special: false,
                        needs_loyalty_card: false,
                    })
                    .collect(),
            })
            .collect();

        Self {
            kind,
            stops,
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
            .into_iter()
            .map(|kind| self.scenario(kind).total_cost())
            .max()?;
        let saving = dearest - self.cheapest.total_cost();
        (saving > Cost::from_cents(0)).then_some(saving)
    }

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

#[derive(Clone, Debug, Default)]
pub enum ResultsState {
    #[default]
    Idle,
    Loading,
    Ready(Box<Results>),
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(store_name: Option<&str>, items: Vec<ItemLine>) -> StoreStop {
        StoreStop {
            store_name: store_name.map(str::to_owned),
            chain: StoreBrand::Paknsave,
            address: Some(String::new()),
            items,
        }
    }

    fn item(name: &str, multiplier: u32, quantity: &str, unit_cents: u32) -> ItemLine {
        ItemLine {
            name: name.to_owned(),
            multiplier,
            quantity: quantity.to_owned(),
            unit_price: Cost::from_cents(unit_cents),
            on_special: false,
            needs_loyalty_card: false,
        }
    }

    fn scenario(kind: ScenarioKind, stops: Vec<StoreStop>) -> Scenario {
        Scenario {
            kind,
            stops,
            travel_cost: Some(Cost::from_cents(380)),
            distance_km: Some(Distance::from_kilometres_f64(8.6)),
            duration_min: Some(19.0),
        }
    }

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
    fn missing_values_format_as_unavailable() {
        assert_eq!(format_or_unavailable(None, format_duration), UNAVAILABLE);
        assert_eq!(format_or_unavailable(Some(19.0), format_duration), "19 min");
    }

    #[test]
    fn total_is_groceries_plus_travel() {
        let scenario = scenario(
            ScenarioKind::Cheapest,
            vec![stop(Some("Test"), vec![item("Milk 2L", 1, "1x 2L", 898)])],
        );
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 1);
    }

    #[test]
    fn line_totals_use_the_pack_multiplier() {
        let scenario = scenario(
            ScenarioKind::Cheapest,
            vec![stop(Some("Test"), vec![item("Milk 1L", 2, "2x 1L", 449)])],
        );
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 2);
    }

    #[test]
    fn store_summary_falls_back_to_the_chain_name() {
        let empty = scenario(ScenarioKind::Best, Vec::new());
        assert_eq!(empty.store_summary(), "No store found");

        let named_and_unnamed = scenario(
            ScenarioKind::Best,
            vec![stop(Some("Test"), Vec::new()), stop(None, Vec::new())],
        );
        assert_eq!(named_and_unnamed.store_summary(), "Test + Pak'nSave");
    }

    #[test]
    fn headline_saving_is_the_gap_to_the_dearest_plan() {
        let milk = |cents| vec![stop(None, vec![item("Milk 1L", 1, "1x 1L", cents)])];
        let results = |best, cheapest, fastest| Results {
            origin_label: String::new(),
            best: scenario(ScenarioKind::Best, milk(best)),
            cheapest: scenario(ScenarioKind::Cheapest, milk(cheapest)),
            fastest: scenario(ScenarioKind::Fastest, milk(fastest)),
            unresolved: Vec::new(),
        };

        assert_eq!(
            results(500, 400, 700).headline_saving(),
            Some(Cost::from_cents(300))
        );
        assert_eq!(results(400, 400, 400).headline_saving(), None);
    }
}
