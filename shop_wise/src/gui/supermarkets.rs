use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use util::store::StoreBrand;

use crate::gui::ShowableWidget;
use crate::price_calculator::results::brand_label;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StorePreference {
    pub store_id: u32,
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
            locations: Vec::new(),
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
    pub fn has_loyalty_card(&self, brand: StoreBrand) -> bool {
        self.chains
            .iter()
            .find(|chain| chain.brand == brand)
            .is_some_and(|chain| chain.has_loyalty_card)
    }
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Supermarkets")
            .on_hover_text("Select the Supermarket chains you want to be included in your search");

        for chain in &mut self.chains {
            let name = brand_label(chain.brand);
            let card = loyalty_label(chain.brand);

            ui.checkbox(&mut chain.checked, name)
                .on_hover_text(format!("Include {name} stores in the search"));

            if !chain.checked {
                continue;
            }

            ui.indent(name, |ui| {
                ui.checkbox(&mut chain.has_loyalty_card, card)
                    .on_hover_text(format!("Do you have {name} loyalty card"));

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
                                .on_hover_text("Untick if you do not want this supermarket chain included in search");
                        }
                    });
            });
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