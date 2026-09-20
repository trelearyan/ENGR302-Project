use std::sync::OnceLock;
use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use util::coordinate::Coordinate;
use util::distance::Distance;
use util::store::StoreBrand;

use crate::gui::ShowableWidget;
use crate::price_calculator::results::brand_label;

#[derive(Clone, Debug, Deserialize)]
pub struct Store {
    pub address: String,
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
    #[serde(rename = "supermarket")]
    pub brand: StoreBrand,
}

impl Store {
    fn coordinate(&self) -> Coordinate {
        Coordinate::from_lat_long_f64(self.latitude, self.longitude)
    }
}

#[derive(Debug, Deserialize)]
struct StoreFile {
    stores: Vec<Store>,
}

const WOOLWORTHS_JSON: &str = include_str!("../../../../shopwise/data/woolworths_locations.json");
const NEWWORLD_JSON: &str = include_str!("../../../../shopwise/data/newworld_locations.json");
const PAKNSAVE_JSON: &str = include_str!("../../../../shopwise/data/paknsave_locations.json");

fn all_stores() -> &'static [Store] {
    static STORES: OnceLock<Vec<Store>> = OnceLock::new();

    STORES.get_or_init(|| {
        let woolworths: StoreFile = serde_json::from_str(WOOLWORTHS_JSON)
            .expect("woolworths_locations.json should be valid");
        let newworld: StoreFile =
            serde_json::from_str(NEWWORLD_JSON).expect("newworld_locations.json should be valid");
        let paknsave: StoreFile =
            serde_json::from_str(PAKNSAVE_JSON).expect("paknsave_locations.json should be valid");

        let mut stores = Vec::new();
        stores.extend(woolworths.stores);
        stores.extend(newworld.stores);
        stores.extend(paknsave.stores);
        stores
    })
}

fn stores_in_range(
    brand: StoreBrand,
    origin: Option<&Coordinate>,
    max_range: &Distance,
) -> Vec<&'static Store> {
    let Some(origin) = origin else {
        return Vec::new();
    };

    let mut found: Vec<&'static Store> = all_stores()
        .iter()
        .filter(|store| store.brand == brand)
        .filter(|store| origin.within_range(store.coordinate(), max_range))
        .collect();

    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChainPreference {
    pub brand: StoreBrand,
    pub checked: bool,
    pub has_loyalty_card: bool,
    pub blacklist: Vec<String>,
}

impl ChainPreference {
    fn new(brand: StoreBrand) -> Self {
        Self {
            brand,
            checked: true,
            has_loyalty_card: false,
            blacklist: Vec::new(),
        }
    }

    fn is_blacklist(&self, store_id: &str) -> bool {
        self.blacklist.iter().any(|id| id == store_id)
    }

    fn set_blacklist(&mut self, store_id: &str, blacklist: bool) {
        if blacklist {
            if !self.is_blacklist(store_id) {
                self.blacklist.push(store_id.to_owned());
            }
        } else {
            self.blacklist.retain(|id| id != store_id);
        }
    }
}

fn loyalty_label(brand: StoreBrand) -> &'static str {
    match brand {
        StoreBrand::Paknsave | StoreBrand::Newworld => "I have a Clubcard",
        StoreBrand::Woolworths => "I have an Everyday Rewards Card",
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupermarketsData {
    pub chains: Vec<ChainPreference>,
}

impl SupermarketsData {
    #[must_use]
    pub fn banned_stores(&self) -> Vec<StoreBrand> {
        self.chains
            .iter()
            .filter(|chain| !chain.checked)
            .map(|chain| chain.brand)
            .collect()
    }

    #[must_use]
    pub fn has_loyalty_card(&self, brand: StoreBrand) -> bool {
        self.chains
            .iter()
            .find(|chain| chain.brand == brand)
            .is_some_and(|chain| chain.has_loyalty_card)
    }

    #[must_use]
    pub fn banned_store_ids(&self) -> Vec<String> {
        self.chains
            .iter()
            .flat_map(|chain| chain.blacklist.iter().cloned())
            .collect()
    }

    pub fn show_in_range(
        &mut self,
        ui: &mut Ui,
        origin: Option<&Coordinate>,
        max_range: &Distance,
    ) {
        ui.heading("Supermarkets")
            .on_hover_text("Select the Supermarket chains you want to be included in your search");

        if origin.is_none() {
            ui.label(
                egui::RichText::new("Set your location to see the stores near you")
                    .small()
                    .weak(),
            );
        }

        let mut clubcard: Option<bool> = None;

        for chain in &mut self.chains {
            let name = brand_label(chain.brand);
            let card = loyalty_label(chain.brand);
            let nearby = stores_in_range(chain.brand, origin, max_range);

            ui.checkbox(&mut chain.checked, name)
                .on_hover_text(format!("Include {name} stores in the search"));

            if !chain.checked {
                continue;
            }

            ui.indent(name, |ui| {
                if ui
                    .checkbox(&mut chain.has_loyalty_card, card)
                    .on_hover_text(format!("Do you have {name}'s loyalty card?"))
                    .changed()
                    && matches!(chain.brand, StoreBrand::Paknsave | StoreBrand::Newworld)
                {
                    clubcard = Some(chain.has_loyalty_card);
                }

                egui::CollapsingHeader::new("Choose locations")
                    .id_salt(name)
                    .show(ui, |ui| {
                        if nearby.is_empty() {
                            let message = if origin.is_none() {
                                "Set your location first"
                            } else {
                                "No stores in range. Try to increase your radius."
                            };
                            ui.label(egui::RichText::new(message).small().weak());
                            return;
                        }

                        for store in nearby {
                            let mut included = !chain.is_blacklist(&store.id);
                            if ui
                                .checkbox(&mut included, &store.name)
                                .on_hover_text(store.address.as_str())
                                .changed()
                            {
                                chain.set_blacklist(&store.id, !included);
                            }
                        }
                    });
            });
        }

        if let Some(value) = clubcard {
            for chain in &mut self.chains {
                if matches!(chain.brand, StoreBrand::Paknsave | StoreBrand::Newworld) {
                    chain.has_loyalty_card = value;
                }
            }
        }
    }
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        self.show_in_range(ui, None, &Distance::from_kilometres_f64(0.0));
    }
}

impl Default for SupermarketsData {
    fn default() -> Self {
        Self {
            chains: vec![
                ChainPreference::new(StoreBrand::Paknsave),
                ChainPreference::new(StoreBrand::Newworld),
                ChainPreference::new(StoreBrand::Woolworths),
            ],
        }
    }
}