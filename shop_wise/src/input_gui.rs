use bigdecimal::BigDecimal;
use derive_more::{Display, IsVariant};
use eframe::egui;
use serde::Deserialize;
use util::search::SearchUnits::{DOLLAR, EACH, GRAM, KILOGRAM, LITRE, MILLILITRE};
use std::collections::HashSet;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::{cell::RefCell, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use urlencoding::encode;
use util::coordinate::Coordinate;
use util::cost::{self, Cost};
use util::search::{ShoppingItemQuery, unit_to_str};
use util::distance::Distance;
use util::store::StoreBrand;

use crate::price_calculator::{self, item_resolver, shopping_list_parser};
use crate::route_planner;
use crate::route_planner::filters::StoreFilters;

#[derive(Default, Clone)]
pub struct LocationState {
    latitude: Option<f64>,
    longitude: Option<f64>,
    // location (what the user sees)
    address: String,
}

impl From<LocationState> for Coordinate {
    fn from(value: LocationState) -> Self {
        if let (Some(latitude), Some(longitude)) = (value.latitude, value.longitude) {
            Coordinate::from_lat_long_f64(latitude, longitude)
        } else {
            Coordinate::default()
        }
    }
}

#[derive(Deserialize)]
struct GeocodeResponse {
    lat: String,
    lon: String,
    display_name: String,
}

// store readable address (after converting from coords to address)
#[derive(Deserialize)]
struct ReverseResponse {
    display_name: String,
}

struct Filters {
    max_range: Distance,
    max_stores: u32,
    include_paknsave: bool,
    include_newworld: bool,
    include_woolies: bool,
}

pub struct MyApp {
    shopping_items: Vec<ShoppingItemQuery>,
    filters: Filters,
    mileage_option: MileageOptions,
    mileage_scratch: String,
    // Shared location state
    location_state: Rc<RefCell<LocationState>>,
}

impl MyApp {
    fn new(
        shopping_items: Vec<ShoppingItemQuery>,
        filters: Filters,
        mileage_option: MileageOptions,
    ) -> Self {
        Self {
            shopping_items,
            filters,
            mileage_option,
            mileage_scratch: Cost::from(MileageOptions::default()).to_string(),

            location_state: Rc::new(RefCell::new(LocationState::default())),
        }
    }
    // for debugging
    pub fn print_filters(&self, ui: &mut egui::Ui) {
        ui.label("=== Filter values ===");
        ui.label(format!(
            "Max Range: {}",
            self.filters.max_range.kilometres()
        ));
        ui.label(format!("Max Stores: {}", self.filters.max_stores));
        ui.label(format!("Pak'nSave: {}", self.filters.include_paknsave));
        ui.label(format!("New World: {}", self.filters.include_newworld));
        ui.label(format!("Woolworths: {}", self.filters.include_woolies));
    }

    /*
    Your shopping list
     */
    pub fn shopping_list_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Your shopping list");
        // if the user clicks + Add Item button, creates an empty ShoppingingItem
        if ui.button("+ Add Item").clicked() {
            self.shopping_items.push(ShoppingItemQuery {
                name: String::new(),
                quantity: 1,
                unit: unit_to_str(EACH).to_string(),
            });
        }

        for (i, item) in &mut self.shopping_items.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut item.name);
                ui.add(egui::DragValue::new(&mut item.quantity));

                egui::ComboBox::from_id_salt(i)
                    .selected_text(&item.unit)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut item.unit, unit_to_str(EACH).to_string(),unit_to_str(EACH));
                        ui.selectable_value(&mut item.unit, unit_to_str(GRAM).to_string(), unit_to_str(GRAM));
                        ui.selectable_value(&mut item.unit, unit_to_str(KILOGRAM).to_string(), unit_to_str(KILOGRAM));
                        ui.selectable_value(&mut item.unit, unit_to_str(MILLILITRE).to_string(), unit_to_str(MILLILITRE));
                        ui.selectable_value(&mut item.unit, unit_to_str(LITRE).to_string(), unit_to_str(LITRE));
                        ui.selectable_value(&mut item.unit, unit_to_str(DOLLAR).to_string(), unit_to_str(DOLLAR));
                    });
            });
        }
    }

    pub fn filters(&mut self, ui: &mut egui::Ui) {
        ui.heading("Filters");

        ui.label("Max Range");
        let mut maxrange = self.filters.max_range.kilometres();
        ui.add(egui::Slider::new(&mut maxrange, 1.0..=50.).text("(km)"));
        self.filters.max_range = Distance::from_kilometres_f64(maxrange);

        ui.label("Max Stores per Trip");
        ui.add(egui::Slider::new(&mut self.filters.max_stores, 1..=10));

        ui.separator();

        ui.label("Supermarkets");
        ui.checkbox(&mut self.filters.include_paknsave, "Pak'nSave");

        ui.checkbox(&mut self.filters.include_newworld, "New World");

        ui.checkbox(&mut self.filters.include_woolies, "Woolworths");
    }

    pub fn mileage(&mut self, ui: &mut egui::Ui) {
        // const {
        //     // Assert all non-custom costs are valid
        //     for a in MileageOptions::iter() {
        //         assert!(BigDecimal::from_str(
        //             a.into::<Cost>().inner().to_plain_string().as_str()
        //         ));
        //     }
        // }

        ui.heading("Mileage");
        ui.horizontal(|ui| {
            ui.label("$");
            ui.add_enabled_ui(self.mileage_option.is_custom(), |ui| {
                let field = ui
                    .add_sized(
                        [40.0, 20.0],
                        egui::TextEdit::singleline(&mut self.mileage_scratch),
                    )
                    .on_hover_text("Enable \"Custom\" to the right to enter a specific value");

                if field.lost_focus() {
                    if let Ok(new_value) =
                        BigDecimal::from_str(self.mileage_scratch.as_str()).map(Cost::new)
                    {
                        self.mileage_option = new_value.into();
                    } else {
                        self.mileage_scratch = Cost::from(self.mileage_option.clone()).to_string();
                    }
                } else if !field.has_focus() {
                    self.mileage_scratch = Cost::from(self.mileage_option.clone()).to_string();
                }
            });
            ui.label("/km");
            egui::ComboBox::from_id_salt("the combobox to select mileage option")
                .width(80.0)
                .truncate()
                .selected_text(self.mileage_option.to_string() + "         ") // spaces needed to have constant size box
                .show_ui(ui, |ui| {
                    for option in MileageOptions::iter() {
                        ui.selectable_value(
                            &mut self.mileage_option,
                            option.clone(),
                            option.to_string().as_str(),
                        );
                    }
                });
        });
    }

    pub fn location(&mut self, ui: &mut egui::Ui) {
        ui.heading("Location");

        if ui.button("Use Current Location").clicked() {
            log::info!("Requesting location...");

            #[cfg(target_arch = "wasm32")]
            self.get_current_location(self.location_state.clone());
        }

        ui.separator();

        let mut state = self.location_state.borrow_mut();

        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut state.address).hint_text("Enter an address"));

            let response = ui.add_enabled(
                !state.address.trim().is_empty(),
                egui::Button::new("Find Location"),
            );

            if response.clicked() {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen_futures::spawn_local;

                    let address = state.address.clone();
                    let state_clone = self.location_state.clone();

                    spawn_local(async move {
                        match MyApp::geocode(&address).await {
                            Ok((lat, lon, display_name)) => {
                                log::info!("User entered location: {}", address);
                                log::info!("Latitude: {}, Longitude: {}", lat, lon);
                                log::info!("Formatted address: {}", display_name);

                                let mut state = state_clone.borrow_mut();

                                if display_name == "Location not found" {
                                    state.latitude = None;
                                    state.longitude = None;
                                    state.address = display_name;
                                } else {
                                    state.latitude = Some(lat);
                                    state.longitude = Some(lon);
                                    state.address = display_name;
                                }
                            }

                            Err(err) => {
                                log::error!("Failed to geocode address: {}", err);
                            }
                        }
                    });
                }
            }
        });
    }

    pub fn search_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Search").clicked() {
                log::info!("the search button is clicked!");
                // First: check if both shopping list and location are not empty

                // Sam
                let res = price_calculator::calculator::calculate(&self.shopping_items);
                println!("{:?}", res);
                //item_resolver(&self.shopping_items);

                // Alex
                // TODO: Fix this up once we have a better format of all the stores and individual location blacklisting
                let mut banned_stores = HashSet::<StoreBrand>::new();

                if !self.filters.include_paknsave {
                    banned_stores.insert(StoreBrand::Paknsave);
                }
                if !self.filters.include_newworld {
                    banned_stores.insert(StoreBrand::Newworld);
                }
                if !self.filters.include_woolies {
                    banned_stores.insert(StoreBrand::Woolworths);
                }

                let mut filters = StoreFilters::builder()
                    .location(
                        <RefCell<LocationState> as Clone>::clone(&self.location_state)
                            .into_inner()
                            .clone()
                            .into(),
                    )
                    .range(self.filters.max_range.clone())
                    .max_store_visits(self.filters.max_stores as usize)
                    .disallow_brands(banned_stores.iter().copied().collect::<Vec<_>>().deref());
                let routes = route_planner::all_possible_routes(&filters.build());
            }
        });
    }

    /*
    Getting the user location
     */
    #[cfg(target_arch = "wasm32")]
    fn get_current_location(&self, state: Rc<RefCell<LocationState>>) {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;
        // get browser window
        let window = web_sys::window().unwrap();

        let navigator = window.navigator();

        let geolocation = navigator.geolocation().unwrap();

        // call back
        let success =
            Closure::<dyn FnMut(web_sys::Position)>::new(move |position: web_sys::Position| {
                use wasm_bindgen_futures::spawn_local;

                log::info!("SUCCESS CALLBACK CALLED"); // debug
                let coords = position.coords();

                let latitude = coords.latitude();
                let longitude = coords.longitude();
                {
                    let mut state = state.borrow_mut();

                    state.latitude = Some(latitude);
                    state.longitude = Some(longitude);
                }
                log::info!("Latitude: {}, Longitude: {}", latitude, longitude); // debug
                let state_clone = state.clone();
                spawn_local(async move {
                    match MyApp::reverse_geocode(latitude, longitude).await {
                        Ok(address) => {
                            log::info!("Address: {address}"); // debug

                            let mut state = state_clone.borrow_mut();
                            state.address = address;
                        }
                        Err(err) => {
                            log::error!("Reverse geocode failed: {err}");

                            let mut state = state_clone.borrow_mut();
                            state.address = "Failed to get address.".to_string();
                        }
                    }
                });
            });

        geolocation
            .get_current_position(success.as_ref().unchecked_ref())
            .unwrap();
        success.forget(); // keep this callback alive
    }

    // translate coords --> readable address
    #[cfg(target_arch = "wasm32")]
    async fn reverse_geocode(latitude: f64, longitude: f64) -> Result<String, reqwest::Error> {
        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={}&lon={}",
            latitude, longitude
        );

        let response = reqwest::Client::new()
            .get(url)
            .header("User-Agent", "ShopWise")
            .send()
            .await?;

        let result: ReverseResponse = response.json().await?;

        Ok(result.display_name)
    }

    #[cfg(target_arch = "wasm32")]
    async fn geocode(address: &str) -> Result<(f64, f64, String), reqwest::Error> {
        let encoded_address = encode(address);

        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2&limit=1",
            encoded_address
        );

        let response = reqwest::Client::new()
            .get(url)
            .header("User-Agent", "ShopWise")
            .send()
            .await?;

        let result: Vec<GeocodeResponse> = response.json().await?;

        if let Some(first) = result.first() {
            let latitude = first.lat.parse::<f64>().unwrap();
            let longitude = first.lon.parse::<f64>().unwrap();

            Ok((latitude, longitude, first.display_name.clone()))
        } else {
            log::error!("Location not found.");

            Ok((0.0, 0.0, "Location not found".to_string()))
        }
    }
}

// https://www.ird.govt.nz/income-tax/income-tax-for-businesses-and-organisations/types-of-business-expenses/claiming-vehicle-expenses/kilometre-rates-2025-2026
#[derive(Debug, Display, EnumIter, IsVariant, Clone, PartialEq, Default)]
enum MileageOptions {
    #[default]
    Petrol,
    Diesel,
    Hybrid,
    Electric,
    #[display("Custom mileage")]
    Custom(Cost),
    #[display("Dont calculate mileage")]
    DontCalculateMileage,
}

impl From<MileageOptions> for Cost {
    fn from(value: MileageOptions) -> Self {
        match value {
            MileageOptions::Petrol => Cost::from_cents(37),
            MileageOptions::Diesel => Cost::from_cents(38),
            MileageOptions::Hybrid => Cost::from_cents(24),
            MileageOptions::Electric => Cost::from_cents(23),
            MileageOptions::Custom(cost) => cost,
            MileageOptions::DontCalculateMileage => Cost::from_cents(0),
        }
    }
}

impl From<Cost> for MileageOptions {
    fn from(value: Cost) -> Self {
        match value {
            x if x == Cost::from_cents(37) => MileageOptions::Petrol,
            x if x == Cost::from_cents(38) => MileageOptions::Diesel,
            x if x == Cost::from_cents(24) => MileageOptions::Hybrid,
            x if x == Cost::from_cents(23) => MileageOptions::Electric,
            x if x == Cost::from_cents(0) => MileageOptions::DontCalculateMileage,
            custom => MileageOptions::Custom(custom),
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new(
            Vec::new(),
            Filters {
                max_range: Distance::from_kilometres_f64(10.),
                max_stores: 3,
                include_paknsave: true,
                include_newworld: true,
                include_woolies: true,
            },
            MileageOptions::default(),
        )
    }
}
