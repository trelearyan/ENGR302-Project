use eframe::egui::{CentralPanel, Panel, ScrollArea, Ui};

use crate::gui::location_search::LocationData;
use crate::gui::output_routes::OutputRoutesData;
use crate::gui::preferences::PreferencesData;
use crate::gui::shopping_list::ShoppingListData;
use crate::gui::supermarkets::SupermarketsData;
use crate::gui::transit::TransitData;

pub mod location_search;
pub mod output_routes;
pub mod preferences;
pub mod shopping_list;
pub mod supermarkets;
pub mod transit;

#[derive(Default, Clone, Debug)]
pub struct AppData {
    shopping_list: ShoppingListData,
    preferences: PreferencesData,
    supermarkets: SupermarketsData,
    transit: TransitData,
    location_search: LocationData,
    output_routes: OutputRoutesData,
}

pub trait ShowableWidget {
    fn show(&mut self, ui: &mut Ui);
}

impl ShowableWidget for AppData {
    fn show(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            Panel::left("left_panel").show_inside(ui, |ui| {
                self.shopping_list.show(ui);
                ui.separator();
                self.preferences.show(ui);
                ui.separator();
                self.supermarkets.show(ui);
                ui.separator();
                self.transit.show(ui);
                ui.separator();
                self.location_search.show(ui);
            });

            CentralPanel::default().show_inside(ui, |ui| {
                self.output_routes.show(ui);
            });
        });
    }
}
