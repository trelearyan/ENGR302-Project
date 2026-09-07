use eframe::egui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::ShowableWidget;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupermarketsData {
    include_paknsave: bool,
    include_newworld: bool,
    include_woolies: bool,
}

impl ShowableWidget for SupermarketsData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Supermarkets");
        ui.checkbox(&mut self.include_paknsave, "Pak'nSave");
        ui.checkbox(&mut self.include_newworld, "New World");
        ui.checkbox(&mut self.include_woolies, "Woolworths");
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
