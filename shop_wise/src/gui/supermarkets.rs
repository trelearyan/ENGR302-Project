use eframe::egui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::ShowableWidget;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupermarketsData {
    pub include_paknsave: bool,
    pub include_newworld: bool,
    pub include_woolworths: bool,
    pub paknsave_loyalty: bool,
    pub newworld_loyalty: bool,
    pub woolworths_loyalty: bool,
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Supermarkets")
            .on_hover_text("Select the chains you wish to be included in your search");

        let chains = [
            (
                &mut self.include_paknsave,
                &mut self.paknsave_loyalty,
                "Pak'nSave",
                "I have a Clubcard",
            ),
            (
                &mut self.include_newworld,
                &mut self.newworld_loyalty,
                "New World",
                "I have a Clubcard",
            ),
            (
                &mut self.include_woolworths,
                &mut self.woolworths_loyalty,
                "Woolworths",
                "I have an Everyday Rewards Card",
            ),
        ];
        for (included, loyalty, name, loyalty_label) in chains {
        ui.checkbox(included, name)
            .on_hover_text(format!("Include {name} stores in the search"));
        if *included {
            ui.indent(name, |ui| {
                ui.checkbox(loyalty, loyalty_label)
                    .on_hover_text("Do you have a Pak'nSave loyalty card");
            });
        }
        }
    }
}

impl Default for SupermarketsData {
    fn default() -> Self {
        Self {
            include_paknsave: true,
            include_newworld: true,
            include_woolworths: true,
            paknsave_loyalty: false,
            newworld_loyalty: false,
            woolworths_loyalty: false,
        }
    }
}