use eframe::egui::{self, CentralPanel, Color32, Panel, ScrollArea, Stroke, Ui};
use std::cell::RefCell;
use std::rc::Rc;
use util::coordinate::Coordinate;


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
pub mod masthead;
pub mod footer;

const SECTION_MAX_HEIGHT: f32 = 300.0;

#[derive(Default, Clone)]
pub struct AppData {
    shopping_list: ShoppingListData,
    preferences: PreferencesData,
    supermarkets: SupermarketsData,
    transit: TransitData,
    location_search: Rc<RefCell<LocationData>>,
    output_routes: OutputRoutesData,
    footer: footer::FooterData,
    pub(crate) filters_collapsed: bool,
}

pub trait ShowableWidget {
    fn show(&mut self, ui: &mut Ui);
}

impl ShowableWidget for AppData {
    fn show(&mut self, ui: &mut Ui) {
        self.masthead(ui);

        if !self.filters_collapsed{
            let panel = Panel::left("left_panel").show_inside(ui, |ui| {
                Panel::bottom("left_footer")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                      self.footer.show(ui);
                    });
                ScrollArea::vertical()
                    .id_salt("left panel scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui|{
                        ScrollArea::vertical()
                            .id_salt("shopping_list_scroll")
                            .max_height(SECTION_MAX_HEIGHT)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                self.shopping_list.show(ui);
                            });
                        ui.separator();
                        self.preferences.show(ui);
                        ui.separator();
                        ScrollArea::vertical()
                            .id_salt("supermarkets_scroll")
                            .max_height(SECTION_MAX_HEIGHT)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                let origin = {
                                    let loc = self.location_search.borrow();
                                    match (loc.latitude, loc.longitude) {
                                        (Some(lat), Some(lon)) => {
                                            Some(Coordinate::from_lat_long_f64(lat, lon))
                                        }
                                        _ => None,
                                    }
                                };
                                self.supermarkets.show_in_range(
                                    ui,
                                    origin.as_ref(),
                                    &self.preferences.max_range,
                                );
                            });
                        ui.separator();
                        self.transit.show(ui);
                        ui.separator();
                    });
            });
            let line_width = 1.0_f32;
            let bar = panel.response.rect;
            ui.painter().vline(bar.right(), bar.y_range(), Stroke::new(line_width, Color32::BLACK));
        }

        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("central panel scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    LocationData::show(&self.location_search, ui);
                    ui.separator();
                    self.search_button(ui);
                    self.output_routes.show(ui);
                })
        });
    }
}

impl AppData {
    pub fn search_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // First: check if both shopping list and location are not empty
            log::debug!("search button called");
            let has_items = self
                .shopping_list
                .shopping_items
                .iter()
                .any(|item| !item.name.trim().is_empty());
            log::debug!("has items {has_items}");
            let location_ready = matches!(
                self.location_search.borrow().status,
                LocationStatus::Resolved | LocationStatus::Success
            );

            let can_search = has_items && location_ready;
            log::debug!("can search {can_search}");

            let mut button = ui.add_enabled(can_search, egui::Button::new("Search"));
            log::debug!("search button");

            if !can_search {
                log::debug!("cant search if");
                let _reason = match (has_items, location_ready) {
                    (false, false) => "Add an item and set a valid location to search",
                    (false, true) => "Add at least one item to your shopping list",
                    (true, false) => {
                        "Choose a location from the suggestions (or use current location)"
                    }
                    (true, true) => unreachable!(),
                };
                button = button.on_disabled_hover_text("Check if you have added an item to shopping list and entered a location");
            }
            if button
                .on_hover_text("I am ready to search")
                .clicked() {
                log::debug!("button clicked");
                // Alex
                // TODO: Fix this up once we have a better format of all the stores and individual location blacklisting
                let banned_stores = self.supermarkets.banned_stores();

                let filters_builder = StoreFilters::builder()
                    .location(Coordinate::from_lat_long_f64(
                        self.location_search.borrow().latitude.unwrap(),
                        self.location_search.borrow().longitude.unwrap(),
                    ))
                    .range(self.preferences.max_range.clone())
                    .max_store_visits(self.preferences.max_stores)
                    .disallow_brands(&banned_stores);
                let filters = filters_builder.build();

                let _origin_label = self.location_search.borrow().address.clone();

                // Sam
                let res = price_calculator::calculate(
                    &self.shopping_list.shopping_items,
                    &filters,
                    &self.transit.mileage_option,
                );
                log::info!("{res:?}");

                let origin_label = self.location_search.borrow().address.clone();
                self.output_routes.results = match res {
                    Ok(calc) => ResultsState::Ready(Box::new(
                        price_calculator::results::Results::from_calculation(&calc, origin_label),
                    )),
                    _ => ResultsState::Failed(
                        res.unwrap_err().pretty(),
                    ),
                };
            }
        });
    }
}
