use crate::results::{
    Chain, ItemLine, Results, Scenario, ScenarioKind, StoreStop, UnresolvedItem, UnresolvedReason,
};

fn item(name: &str, quantity: u32, unit_price: i64) -> ItemLine {
    ItemLine {
        name: name.to_owned(),
        quantity,
        unit_price,
        on_special: false,
        needs_loyalty_card: false,
    }
}

fn special(name: &str, quantity: u32, unit_price: i64) -> ItemLine {
    ItemLine { on_special: true, ..item(name, quantity, unit_price) }
}

fn club(name: &str, quantity: u32, unit_price: i64) -> ItemLine {
    ItemLine { needs_loyalty_card: true, ..item(name, quantity, unit_price) }
}

pub fn demo_results() -> Results {
    let paknsave = |items: Vec<ItemLine>| StoreStop {
        store_name: "Pak'nSave Kilbirnie".to_owned(),
        chain: Chain::PakNSave,
        address: "8 Ross Street, Kilbirnie".to_owned(),
        items,
    };
    let new_world = |items: Vec<ItemLine>| StoreStop {
        store_name: "New World Thorndon".to_owned(),
        chain: Chain::NewWorld,
        address: "279 Thorndon Quay, Pipitea".to_owned(),
        items,
    };
    let woolworths = |items: Vec<ItemLine>| StoreStop {
        store_name: "Woolworths Chaffers".to_owned(),
        chain: Chain::Woolworths,
        address: "279 Wakefield Street, Te Aro".to_owned(),
        items,
    };

    let cheapest = Scenario {
        kind: ScenarioKind::Cheapest,
        stops: vec![
            paknsave(vec![
                item("Anchor Blue Milk 2L", 2, 449),
                item("Vogel's Original Mixed Grain 720g", 1, 559),
                special("Tip Top Vanilla Ice Cream 2L", 1, 599),
                item("Bananas, loose, per kg", 2, 349),
                item("Pams Spaghetti 500g", 3, 129),
            ]),
            new_world(vec![
                club("Free range eggs, size 7, dozen", 1, 899),
                item("Chicken breast, skinless, per kg", 1, 1699),
            ]),
        ],
        travel_cost: 642,
        distance_km: 14.8,
        duration_min: 31.0,
    };

    let best = Scenario {
        kind: ScenarioKind::Best,
        stops: vec![paknsave(vec![
            item("Anchor Blue Milk 2L", 2, 449),
            item("Vogel's Original Mixed Grain 720g", 1, 559),
            special("Tip Top Vanilla Ice Cream 2L", 1, 599),
            item("Bananas, loose, per kg", 2, 349),
            item("Pams Spaghetti 500g", 3, 129),
            item("Free range eggs, size 7, dozen", 1, 979),
            item("Chicken breast, skinless, per kg", 1, 1799),
        ])],
        travel_cost: 381,
        distance_km: 8.6,
        duration_min: 19.0,
    };

    let fastest = Scenario {
        kind: ScenarioKind::Fastest,
        stops: vec![woolworths(vec![
            item("Anchor Blue Milk 2L", 2, 499),
            item("Vogel's Original Mixed Grain 720g", 1, 629),
            item("Tip Top Vanilla Ice Cream 2L", 1, 749),
            item("Bananas, loose, per kg", 2, 399),
            item("Pams Spaghetti 500g", 3, 159),
            club("Free range eggs, size 7, dozen", 1, 1049),
            item("Chicken breast, skinless, per kg", 1, 1899),
        ])],
        travel_cost: 187,
        distance_km: 4.2,
        duration_min: 11.0,
    };

    Results {
        origin_label: "12 Kelburn Parade, Kelburn, Wellington".to_owned(),
        best,
        cheapest,
        fastest,
        unresolved: vec![
            UnresolvedItem {
                raw_text: "avacado".to_owned(),
                reason: UnresolvedReason::NotRecognised,
            },
            UnresolvedItem {
                raw_text: "ginger kombucha 1L".to_owned(),
                reason: UnresolvedReason::NotStockedNearby,
            },
        ],
    }
}

pub fn demo_results_without_warnings() -> Results {
    Results { unresolved: Vec::new(), ..demo_results() }
}
