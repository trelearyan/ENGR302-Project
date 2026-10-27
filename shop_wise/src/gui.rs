use eframe::egui::{CentralPanel, Panel, ScrollArea, Ui};
use std::cell::RefCell;
use std::rc::Rc;
use util::coordinate::Coordinate;

use crate::gui::location_search::LocationData;
use crate::gui::output_routes::OutputRoutesData;
use crate::gui::preferences::PreferencesData;
use crate::gui::shopping_list::ShoppingListData;
use crate::gui::supermarkets::SupermarketsData;
use crate::gui::transit::TransitData;
use crate::price_calculator::results::ResultsState;
use crate::price_calculator::{self};
use crate::route_planner::filters::StoreFilters;

pub mod location_search;
pub mod masthead;
pub mod output_routes;
pub mod preferences;
pub mod shopping_list;
pub mod supermarkets;
pub mod transit;

const SECTION_MAX_HEIGHT: f32 = 300.0;

#[derive(Default, Clone)]
pub struct AppData {
    shopping_list: ShoppingListData,
    preferences: PreferencesData,
    supermarkets: SupermarketsData,
    transit: TransitData,
    location_search: Rc<RefCell<LocationData>>,
    output_routes: OutputRoutesData,
    pub(crate) filters_collapsed: bool,
}

pub trait ShowableWidget {
    fn show(&mut self, ui: &mut Ui);
}

impl ShowableWidget for AppData {
    fn show(&mut self, ui: &mut Ui) {
        self.masthead(ui);

        if !self.filters_collapsed {
            Panel::left("left_panel").show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt("left panel scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
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
        }

        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("central panel scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let has_items = self
                        .shopping_list
                        .shopping_items
                        .iter()
                        .any(|item| !item.name.trim().is_empty());
                    if LocationData::show(&self.location_search, ui, has_items) {
                        self.search();
                    }
                    ui.add_space(12.0);
                    self.output_routes.show(ui);
                })
        });
    }
}

impl AppData {
    pub fn search(&mut self) {
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
                price_calculator::results::Results::from_calculation(
                    &calc,
                    origin_label,
                    &self.output_routes.graph,
                ),
            )),
            _ => ResultsState::Failed(res.unwrap_err().pretty()),
        };
    }
}
