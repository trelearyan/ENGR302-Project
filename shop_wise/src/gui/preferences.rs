use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use util::distance::Distance;

use crate::gui::ShowableWidget;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesData {
    max_range: Distance,
    max_stores: u32,
}

impl ShowableWidget for PreferencesData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Preferences");

        ui.label("Max Range");
        let mut maxrange = self.max_range.kilometres();
        ui.add(egui::Slider::new(&mut maxrange, 1.0..=50.).text("(km)"));
        self.max_range = Distance::from_kilometres_f64(maxrange);

        ui.label("Max Stores per Trip");
        ui.add(egui::Slider::new(&mut self.max_stores, 1..=10));
    }
}

impl Default for PreferencesData {
    fn default() -> Self {
        Self {
            max_range: Distance::from_kilometres_f64(25.0),
            max_stores: 3,
        }
    }
}
