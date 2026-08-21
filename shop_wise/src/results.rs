use bigdecimal::{BigDecimal, Zero};
use util::cost::Cost;
use util::distance::Distance;
use util::store::StoreBrand;
pub fn format_money(cost: &Cost) -> String {
    format!("${cost}")
}

pub fn format_distance(distance: &Distance) -> String {
    format!("{:.1} km", distance.kilometres())
}

pub fn format_duration(minutes: f32) -> String {
    let total = minutes.round().max(0.0) as u32;
    if total < 60 {
        format!("{total} min")
    } else {
        format!("{} h {:02} min", total / 60, total % 60)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScenarioKind {
    Best,
    Cheapest,
    Fastest,
}

impl ScenarioKind {
    pub const ALL: [ScenarioKind; 3] = [ScenarioKind::Best, ScenarioKind::Cheapest, ScenarioKind::Fastest];

    pub fn label(self) -> &'static str {
        match self {
            ScenarioKind::Best => "Best value",
            ScenarioKind::Cheapest => "Cheapest",
            ScenarioKind::Fastest => "Fastest",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            ScenarioKind::Best => "Balances money saved against time spent",
            ScenarioKind::Cheapest => "Lowest groceries plus petrol",
            ScenarioKind::Fastest => "Shortest trip that still gets everything",
        }
    }
}

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
    pub quantity: u32,
    pub unit_price: Cost,
    pub on_special: bool,
    pub needs_loyalty_card: bool,
}

impl ItemLine {
    pub fn line_total(&self) -> Cost {
        self.unit_price.clone() * BigDecimal::from(self.quantity)
    }
}

#[derive(Clone, Debug)]
pub struct StoreStop {
    pub store_name: String,
    pub chain: StoreBrand,
    pub address: String,
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
    pub travel_cost: Cost,
    pub distance_km: Distance,
    pub duration_min: f32,
}

impl Scenario {
    pub fn grocery_cost(&self) -> Cost {
        self.stops.iter().map(StoreStop::subtotal).sum()
    }

    pub fn total_cost(&self) -> Cost {
        self.grocery_cost() + self.travel_cost.clone()
    }
    pub fn item_count(&self) -> u32 {
        self.stops.iter().flat_map(|s| s.items.iter()).map(|i| i.quantity).sum()
    }

    pub fn store_summary(&self) -> String {
        if self.stops.is_empty() {
            return "No store found".to_owned();
        }
        self.stops
            .iter()
            .map(|s| s.store_name.as_str())
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnresolvedReason {
    NotRecognised,
    NotStockedNearby,
}

impl UnresolvedReason {
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
    pub fn scenario(&self, kind: ScenarioKind) -> &Scenario {
        match kind {
            ScenarioKind::Best => &self.best,
            ScenarioKind::Cheapest => &self.cheapest,
            ScenarioKind::Fastest => &self.fastest,
        }
    }

    pub fn headline_saving(&self) -> Option<Cost> {
        let dearest = ScenarioKind::ALL.iter().map(|k| self.scenario(*k).total_cost()).max()?;
        let saving = dearest - self.cheapest.total_cost();
        (saving.clone().inner() > BigDecimal::zero()).then_some(saving)
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
            store_name: "Test".to_owned(),
            chain: StoreBrand::Paknsave,
            address: String::new(),
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
            travel_cost: Cost::from_cents(380),
            distance_km: Distance::from_kilometres_f64(8.6),
            duration_min: 19.0,
        };
        assert_eq!(scenario.grocery_cost(), Cost::from_cents(898));
        assert_eq!(scenario.total_cost(), Cost::from_cents(1278));
        assert_eq!(scenario.item_count(), 2);
    }
}
