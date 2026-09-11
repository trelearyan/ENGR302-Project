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

        ui.checkbox(&mut self.include_paknsave, "Pak'nSave")
            .on_hover_text("Include Pak'nSave stores in the comparison");
        if self.include_paknsave {
            ui.indent("paknsave_options", |ui| {
                ui.checkbox(&mut self.paknsave_loyalty, "I have a Clubcard")
                    .on_hover_text("Do you have a Pak'nSave loyalty card");
            });
        }

        ui.checkbox(&mut self.include_newworld, "New World")
            .on_hover_text("Include New World stores in the comparison");
        if self.include_newworld {
            ui.indent("newworld_options", |ui| {
                ui.checkbox(&mut self.newworld_loyalty, "I have a Clubcard")
                    .on_hover_text("Do you have a New World loyalty card");
            });
        }

        ui.checkbox(&mut self.include_woolworths, "Woolworths")
            .on_hover_text("Include Woolworths stores in the comparison");
        if self.include_woolworths {
            ui.indent("woolworths_options", |ui| {
                ui.checkbox(&mut self.woolworths_loyalty, "I have an Everyday Rewards Card")
                    .on_hover_text("Do you have a Woolworths loyalty card");
            });
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