use eframe::egui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::ShowableWidget;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupermarketsData {
    pub include_paknsave: bool,
    pub include_newworld: bool,
    pub include_woolies: bool,
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Supermarkets")
            .on_hover_text("Select the chains you wish to be included in your search");
        ui.checkbox(&mut self.include_paknsave, "Pak'nSave")
            .on_hover_text("Include Pak'nSave stores in the comparison");
        ui.checkbox(&mut self.include_newworld, "New World")
            .on_hover_text("Include New World stores in the comparison");
        ui.checkbox(&mut self.include_woolies, "Woolworths")
            .on_hover_text("Include Woolworths stores in the comparison");
    }
}

impl Default for SupermarketsData {
    fn default() -> Self {
        Self {
            include_paknsave: true,
            include_newworld: true,
            include_woolies: true,
        }
    }
}
