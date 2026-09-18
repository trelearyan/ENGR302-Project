use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
struct StoreFile {
    stores: Vec<Store>,
}

const WOOLWORTHS_JSON: &str = include_str!("../../../../shopwise/data/woolworths_locations.json");
const NEWWORLD_JSON: &str = include_str!("../../../../shopwise/data/newworld_locations.json");
const PAKNSAVE_JSON: &str = include_str!("../../../../shopwise/data/paknsave_locations.json");

#[must_use]
fn all_stores() -> Vec<Store> {
    let mut stores = Vec::new();

    let woolworths: StoreFile =
        serde_json::from_str(WOOLWORTHS_JSON)
            .expect("woolworths_locations.json should be valid");

    let newworld: StoreFile =
        serde_json::from_str(NEWWORLD_JSON)
            .expect("newworld_locations.json should be valid");

    let paknsave: StoreFile =
        serde_json::from_str(PAKNSAVE_JSON)
            .expect("paknsave_locations.json should be valid");

    stores.extend(woolworths.stores);
    stores.extend(newworld.stores);
    stores.extend(paknsave.stores);

    stores
}

#[must_use]
fn stores_for(brand: StoreBrand) -> Vec<Store> {
    all_stores()
        .into_iter()
        .filter(|store| store.brand == brand)
        .collect()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StorePreference {
    pub store_id: String,
    pub name: String,
    pub checked: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChainPreference {
    pub brand: StoreBrand,
    pub checked: bool,
    pub has_loyalty_card: bool,
    pub locations: Vec<StorePreference>,
}

impl ChainPreference {
    fn new(brand: StoreBrand) -> Self {
        Self {
            brand,
            checked: true,
            has_loyalty_card: false,
            locations: stores_for(brand)
                .into_iter()
                .map(|store| StorePreference {
                    store_id: store.id,
                    name: store.name,
                    checked: true,
                })
                .collect(),
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
            .flat_map(|chain| chain.locations.iter())
            .filter(|store| !store.checked)
            .map(|store| store.store_id.clone())
            .collect()
    }
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Supermarkets")
            .on_hover_text(
                "Select the Supermarket chains you want to be included in your search",
            );

        let mut clubcard: Option<bool> = None;

        for chain in &mut self.chains {
            let name = brand_label(chain.brand);
            let card = loyalty_label(chain.brand);

            ui.checkbox(&mut chain.checked, name)
                .on_hover_text(format!("Include {name} stores in the search"));

            if !chain.checked {
                continue;
            }

            ui.indent(name, |ui| {
                if ui
                    .checkbox(&mut chain.has_loyalty_card, card)
                    .on_hover_text(format!(
                        "Do you have {name}'s loyalty card?"
                    ))
                    .changed()
                    && matches!(
                        chain.brand,
                        StoreBrand::Paknsave | StoreBrand::Newworld
                    )
                {
                    clubcard = Some(chain.has_loyalty_card);
                }

                egui::CollapsingHeader::new("Choose locations")
                    .id_salt(name)
                    .show(ui, |ui| {
                        if chain.locations.is_empty() {
                            ui.label(
                                egui::RichText::new("not available yet")
                                    .small()
                                    .weak(),
                            );
                            return;
                        }

                        for store in &mut chain.locations {
                            ui.checkbox(&mut store.checked, &store.name)
                                .on_hover_text(
                                    "Untick if you do not want this supermarket chain included in search",
                                );
                        }
                    });
            });
        }

        if let Some(value) = clubcard {
            for chain in &mut self.chains {
                if matches!(
                    chain.brand,
                    StoreBrand::Paknsave | StoreBrand::Newworld
                ) {
                    chain.has_loyalty_card = value;
                }
            }
        }
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