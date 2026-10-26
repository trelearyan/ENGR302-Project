use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use util::distance::Distance;

use crate::gui::ShowableWidget;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesData {
    pub max_range: Distance,
    pub search_all_stores: bool,
    pub max_stores: usize,
}

impl PreferencesData {
    /// The search radius, or `None` to search every store.
    #[must_use]
    pub fn range(&self) -> Option<&Distance> {
        (!self.search_all_stores).then_some(&self.max_range)
    }
}

impl ShowableWidget for PreferencesData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Preferences");

        ui.label("Max Range")
            .on_hover_text("How far would you want to travel from your location");
        let mut maxrange = self.max_range.kilometres();
        ui.add_enabled(
            !self.search_all_stores,
            egui::Slider::new(&mut maxrange, 1.0..=50.).text("(km)"),
        )
        .on_hover_text("Use the slider to pick up your range");
        self.max_range = Distance::from_kilometres_f64(maxrange);
        ui.checkbox(&mut self.search_all_stores, "Search all stores")
            .on_hover_text("Ignore the max range and include every store, however far away");

        ui.label("Max Stores per Trip")
            .on_hover_text("Select the amount of stores to place your range");
        ui.add(egui::Slider::new(&mut self.max_stores, 1..=10))
            .on_hover_text("Move the slider to select your maximum stores");
    }
}

impl Default for PreferencesData {
    fn default() -> Self {
        Self {
            max_range: Distance::from_kilometres_f64(25.0),
            search_all_stores: false,
            max_stores: 3,
        }
    }
}
