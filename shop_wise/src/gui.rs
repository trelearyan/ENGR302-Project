use eframe::CreationContext;
use eframe::egui::{CentralPanel, Panel, ScrollArea, Ui};
use std::cell::RefCell;
use std::rc::Rc;
use util::coordinate::Coordinate;

use crate::gui::location_search::LocationData;
use crate::gui::map::MapData;
use crate::gui::output_routes::OutputRoutesData;
use crate::gui::preferences::PreferencesData;
use crate::gui::shopping_list::ShoppingListData;
use crate::gui::supermarkets::SupermarketsData;
use crate::gui::transit::TransitData;
use crate::price_calculator::results::ResultsState;
use crate::price_calculator::{self};
use crate::route_planner::filters::StoreFilters;

pub mod footer;
pub mod location_search;
pub mod masthead;
pub mod output_routes;
pub mod preferences;
pub mod shopping_list;
pub mod supermarkets;
pub mod transit;

pub mod map {
    use std::sync::Arc;

    use eframe::{
        CreationContext,
        egui::{self, CentralPanel, Color32},
        epaint::PathStroke,
    };
    use util::coordinate::Coordinate;
    use walkers::{HttpTiles, Map, MapMemory, Plugin, Position, Projector, sources::OpenStreetMap};

    use crate::gui::ShowableWidget;

    pub struct MapData {
        tiles: HttpTiles,
        memory: MapMemory,
        pub route: Arc<[Coordinate]>,
    }

    impl ShowableWidget for MapData {
        fn show(&mut self, ui: &mut egui::Ui) {
            CentralPanel::default().show(ui, |ui| {
                let mut map = Map::new(
                    Some(&mut self.tiles),
                    &mut self.memory,
                    walkers::lon_lat(174.77643827862266, -41.288175480492), // Wellington
                );
                if self.route.len() > 1 {
                    log::debug!(
                        "Route length greater than 1 at {:?}, showing route",
                        self.route.len()
                    );
                    map = map.with_plugin(RouteDrawingPlugin(self.route.clone()));
                }
                log::debug!(
                    "Route length less than 1, is {:?}, no show route",
                    self.route
                );
                ui.add(map);
            });
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct RouteDrawingPlugin(Arc<[Coordinate]>);

    impl Plugin for RouteDrawingPlugin {
        fn run(
            self: Box<Self>,
            ui: &mut egui::Ui,
            _response: &egui::Response,
            projector: &Projector,
            _map_memory: &MapMemory,
        ) {
            ui.painter().line(
                self.0
                    .iter()
                    .map(|coordinate| {
                        projector
                            .project(Position::from(coordinate.clone()))
                            .to_pos2()
                    })
                    .collect(),
                PathStroke::new(3.0, Color32::RED),
            );
        }
    }

    impl MapData {
        pub fn new(ctx: &CreationContext) -> Self {
            Self {
                tiles: HttpTiles::new(OpenStreetMap, ctx.egui_ctx.clone()),
                memory: MapMemory::default(),
                route: Arc::new([]),
            }
        }
    }
}

const SECTION_MAX_HEIGHT: f32 = 300.0;

pub struct AppData {
    shopping_list: ShoppingListData,
    preferences: PreferencesData,
    supermarkets: SupermarketsData,
    transit: TransitData,
    location_search: Rc<RefCell<LocationData>>,
    output_routes: OutputRoutesData,
    map: MapData,
    footer: footer::FooterData,
    pub(crate) filters_collapsed: bool,
}

impl AppData {
    pub fn new(ctx: &CreationContext) -> Self {
        Self {
            map: MapData::new(ctx),
            shopping_list: Default::default(),
            preferences: Default::default(),
            supermarkets: Default::default(),
            transit: Default::default(),
            location_search: Default::default(),
            output_routes: Default::default(),
            footer: Default::default(),
            filters_collapsed: Default::default(),
        }
    }
}

pub trait ShowableWidget {
    fn show(&mut self, ui: &mut Ui);
}

impl ShowableWidget for AppData {
    fn show(&mut self, ui: &mut Ui) {
        self.masthead(ui);

        if !self.filters_collapsed {
            const LEFT_PANEL_WIDTH: f32 = 300.0;
            let panel = Panel::left("left_panel")
                .default_size(LEFT_PANEL_WIDTH)
                .show(ui, |ui| {
                    Panel::bottom("left_footer")
                        .resizable(false)
                        .show(ui, |ui| {
                            self.footer.show(ui);
                        });
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

            Panel::bottom("bottom").show(ui, |ui| {
                self.footer.show(ui);
            });
        };

        CentralPanel::default().show(ui, |ui| {
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
                    ui.add_space(12.0);
                    self.map.show(ui);
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
            Ok(calc) => {
                match self.output_routes.selected {
                    price_calculator::results::ScenarioKind::Best => {
                        self.map.route = calc.best.source.expensive_calculated_route.clone();
                    }
                    price_calculator::results::ScenarioKind::Cheapest => {
                        self.map.route = calc.cheapest.source.expensive_calculated_route.clone();
                    }
                    price_calculator::results::ScenarioKind::Fastest => {
                        self.map.route = calc.fastest.source.expensive_calculated_route.clone();
                    }
                }
                ResultsState::Ready(Box::new(
                    price_calculator::results::Results::from_calculation(
                        &calc,
                        origin_label,
                        &self.output_routes.graph,
                    ),
                ))
            }
            _ => ResultsState::Failed(res.unwrap_err().pretty()),
        };
    }
}
