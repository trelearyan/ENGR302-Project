use eframe::egui::{self, CentralPanel, Panel, ScrollArea, Ui};
use std::collections::HashSet;
use std::ops::Deref;
use util::coordinate::Coordinate;
use util::store::StoreBrand;

use crate::gui::location_search::{LocationData, LocationStatus};
use crate::gui::output_routes::OutputRoutesData;
use crate::gui::preferences::PreferencesData;
use crate::gui::shopping_list::ShoppingListData;
use crate::gui::supermarkets::SupermarketsData;
use crate::gui::transit::TransitData;
use crate::price_calculator::results::ResultsState;
use crate::price_calculator::{self};
use crate::route_planner::filters::StoreFilters;

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
                self.search_button(ui);
                self.output_routes.show(ui);
            });
        });
    }
}

impl AppData {
    pub fn search_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // First: check if both shopping list and location are not empty
            let has_items = self
                .shopping_list
                .shopping_items
                .iter()
                .any(|item| !item.name.trim().is_empty());
            let location_ready = matches!(
                self.location_search.status,
                LocationStatus::Resolved | LocationStatus::Success
            );

            let can_search = has_items && location_ready;

            let mut button = ui.add_enabled(can_search, egui::Button::new("Search"));

            if !can_search {
                let reason = match (has_items, location_ready) {
                    (false, false) => "Add an item and set a valid location to search",
                    (false, true) => "Add at least one item to your shopping list",
                    (true, false) => {
                        "Choose a location from the suggestions (or use current location)"
                    }
                    (true, true) => unreachable!(),
                };
                button = button.on_disabled_hover_text(reason);
            }
            if button.clicked() {
                // Alex
                // TODO: Fix this up once we have a better format of all the stores and individual location blacklisting
                let mut banned_stores = HashSet::<StoreBrand>::new();

                if !self.supermarkets.include_paknsave {
                    banned_stores.insert(StoreBrand::Paknsave);
                }
                if !self.supermarkets.include_newworld {
                    banned_stores.insert(StoreBrand::Newworld);
                }
                if !self.supermarkets.include_woolies {
                    banned_stores.insert(StoreBrand::Woolworths);
                }

                let filters_builder = StoreFilters::builder()
                    // TODO: fix panic if no configured location
                    .location(Coordinate::from_lat_long_f64(
                        self.location_search.latitude.unwrap(),
                        self.location_search.longitude.unwrap(),
                    ))
                    .range(self.preferences.max_range.clone())
                    .max_store_visits(self.preferences.max_stores)
                    .disallow_brands(banned_stores.iter().copied().collect::<Vec<_>>().deref());
                let filters = filters_builder.build();

                let _origin_label = self.location_search.address.clone();

                // Sam
                let res = price_calculator::calculate(
                    &self.shopping_list.shopping_items,
                    &filters,
                    &self.transit.mileage_option,
                );
                log::info!("{res:?}");

                let origin_label = self.location_search.address.clone();
                self.output_routes.results = match res {
                    Some(calc) => ResultsState::Ready(Box::new(
                        price_calculator::results::Results::from_calculation(&calc, origin_label),
                    )),
                    None => ResultsState::Failed(
                        "No combination of stores in range can supply this list".to_owned(),
                    ),
                };
            }
        });
    }
}
