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
use crate::output::OutputPanel;
use crate::results::ResultsState;

#[derive(Default, Clone, PartialEq)]
pub enum LocationStatus {
    #[default]
    Idle,
    Searching,
    Suggesting,
    NotFound,
    Resolved,

    Locating,
    Success,
    Error(String),
}

#[derive(Default, Clone)]
pub struct LocationState {
    latitude: Option<f64>,
    longitude: Option<f64>,
    // location (what the user sees)
    address: String,
    status: LocationStatus,
    suggestions: Vec<AddressSuggestion>,

    request_id: u64,
    last_edit_time: Option<f64>,
}

// A single resolved candidate shown in the suggestions dropdown.
#[derive(Clone)]
struct AddressSuggestion {
    display_name: String,
    lat: f64,
    lon: f64,
}
 
const DEBOUNCE_SECONDS: f64 = 0.4;
const MIN_QUERY_LEN: usize = 3;

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
    // fr09 output panel
    output: OutputPanel,
    results: ResultsState,
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
            output: OutputPanel::new(),
            results: ResultsState::Idle,
        }
    }

    //fr09 output
    pub fn output_panel(&mut self, ui: &mut egui::Ui) {
        self.output.show(ui, &self.results);
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

                routes
                    .iter()
                    .for_each(|route| println!("{}", route.pretty_print()));

                let origin_label = self.location_state.borrow().address.clone();
                self.results = match price_calculator::calculator::calculate(&self.shopping_items) {
                    Some(calc) => ResultsState::Ready(Box::new(
                        crate::results::Results::from_calculation(&calc, origin_label),
                    )),
                    None => ResultsState::Failed(
                        "No combination of stores in range can supply this list".to_owned(),
                    ),
                };
            }
        });
    }

    /// Location panel with live autocomplete.
    pub fn location(&mut self, ui: &mut egui::Ui) {
        ui.heading("Location");
 
        if ui.button("Use Current Location").clicked() {
            log::info!("Requesting location...");
            #[cfg(target_arch = "wasm32")]
            self.get_current_location(self.location_state.clone(), ui.ctx().clone());
        }
 
        ui.separator();
 
        let now = ui.input(|i| i.time);
        let mut state = self.location_state.borrow_mut();
 
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.address)
                .hint_text("e.g. 12 Example Street, Suburb, City"),
        );
 
        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
 
        if response.changed() {
            state.status = LocationStatus::Idle;
            state.suggestions.clear();
            state.latitude = None;
            state.longitude = None;
            state.last_edit_time = Some(now);
            ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(DEBOUNCE_SECONDS));
        }
 
        // on Enter, Fire a search once the debounce window has elapsed, or immediately
        let should_fire = match state.last_edit_time {
            Some(last_edit) if enter_pressed => true,
            Some(last_edit) if now - last_edit >= DEBOUNCE_SECONDS => true,
            _ => false,
        };
 
        if should_fire {
            let query = state.address.trim().to_string();
            state.last_edit_time = None;
 
            if query.len() < MIN_QUERY_LEN {
                state.status = LocationStatus::Idle;
            } else {
                state.status = LocationStatus::Searching;
                state.request_id += 1;
                let this_request = state.request_id;
                let state_clone = self.location_state.clone();
                let ctx_clone = ui.ctx().clone();
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen_futures::spawn_local;
                    spawn_local(async move {
                        match MyApp::geocode_suggestions(&query).await {
                            Ok(results) => {
                                let mut state = state_clone.borrow_mut();
                                if state.request_id == this_request {
                                    if results.is_empty() {
                                        state.status = LocationStatus::NotFound;
                                    } else {
                                        state.suggestions = results;
                                        state.status = LocationStatus::Suggesting;
                                    }
                                }
                            }
                            Err(err) => {
                                log::error!("Geocode search failed: {}", err);
                                let mut state = state_clone.borrow_mut();
                                if state.request_id == this_request {
                                    state.status = LocationStatus::NotFound;
                                }
                            }
                        }
                        ctx_clone.request_repaint(); 
                    });
                }
            }
        }
 
        match &state.status {
            LocationStatus::Searching => { ui.label("Searching…");}
            LocationStatus::NotFound => {
                ui.colored_label(
                egui::Color32::from_rgb(200, 60, 60),
                "No matching address found — try adding your suburb or city.",
                );
            }
            LocationStatus::Resolved | LocationStatus::Success => { ui.colored_label(egui::Color32::from_rgb(60, 160, 60), "Location set");}
            LocationStatus::Locating => { ui.label("Finding your current location…");}
            LocationStatus::Error(message) => { ui.colored_label(egui::Color32::from_rgb(200, 60, 60), message.clone());}
            LocationStatus::Idle | LocationStatus::Suggesting => {}
        }
 
        // suggestions dropdown 
        if state.status == LocationStatus::Suggesting {
            let suggestions = state.suggestions.clone();
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                for s in &suggestions {
                    if ui.selectable_label(false, &s.display_name).clicked() {
                        state.address = s.display_name.clone();
                        state.latitude = Some(s.lat);
                        state.longitude = Some(s.lon);
                        state.status = LocationStatus::Resolved;
                        state.suggestions.clear();
                        state.last_edit_time = None;
                    }
                }
            });
        }
    }

    /*
    Getting the user location
    */
    #[cfg(target_arch = "wasm32")]
    fn get_current_location(&self, state: Rc<RefCell<LocationState>>, ctx: egui::Context) {
        use wasm_bindgen::JsCast;
 
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let navigator = window.navigator();
 
        let geolocation = match Self::resolve_geolocation(&navigator, &state, &ctx) {
            Some(g) => g,
            None => return,
        };
 
        // egui only repaints in response to input events by default without this, a state change made from a background/async callback might not actually appear on screen until the user happens to move the mouse.
        state.borrow_mut().status = LocationStatus::Locating;
        ctx.request_repaint();
 
        // Options control "browser hangs forever" failure mode:
        // - timeout: give up after 10s instead of waiting indefinitely
        // - maximum_age: accept a cached fix up to 60s old so a repeat click returns near-instantly
        let mut options = web_sys::PositionOptions::new();
        options.set_timeout(10_000);
        options.set_maximum_age(60_000);
 
        let success = Self::make_success_callback(state.clone(), ctx.clone());
        let error = Self::make_error_callback(state.clone(), ctx.clone());
 
        let request = geolocation.get_current_position_with_error_callback_and_options(
            success.as_ref().unchecked_ref(),
            Some(error.as_ref().unchecked_ref()),
            &options,
        );
 
        if let Err(err) = request {
            log::error!("Failed to request geolocation: {:?}", err);
            state.borrow_mut().status = LocationStatus::Error(
                "Couldn't start the location lookup. Please enter your address below instead."
                    .to_string(),
            );
            ctx.request_repaint();
        }
 
        // Keep both callbacks alive for as long as the JS side might call them.
        success.forget();
        error.forget();
    }

    /*
    Gets the Geolocation handle, or records why it isn't available.
    */ 
    #[cfg(target_arch = "wasm32")]
    fn resolve_geolocation(navigator: &web_sys::Navigator,state: &Rc<RefCell<LocationState>>,ctx: &egui::Context) -> Option<web_sys::Geolocation> {
        match navigator.geolocation() {
            Ok(g) => Some(g),
            Err(_) => {
                state.borrow_mut().status = LocationStatus::Error(
                    "Geolocation isn't available. This usually means the page \
                     isn't served over HTTPS, or your browser doesn't support it.".to_string(),
                );
                ctx.request_repaint();
                None
            }
        }
    }

    /*
    Builds the callback fired when the browser successfully returns a position:
    - stores the coordinates, 
    - then kicks off reverse-geocoding in the background
     */
    #[cfg(target_arch = "wasm32")]
    fn make_success_callback(state: Rc<RefCell<LocationState>>,ctx: egui::Context) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Position)> {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen_futures::spawn_local;
 
        Closure::<dyn FnMut(web_sys::Position)>::new(move |position: web_sys::Position| {
            let coords = position.coords();
            let latitude = coords.latitude();
            let longitude = coords.longitude();
            log::info!("Latitude: {}, Longitude: {}", latitude, longitude);
 
            {
                let mut s = state.borrow_mut();
                s.latitude = Some(latitude);
                s.longitude = Some(longitude);
                s.status = LocationStatus::Success;
            }
            ctx.request_repaint();
 
            let state_clone = state.clone();
            let ctx_clone = ctx.clone();
            spawn_local(async move {
                match MyApp::reverse_geocode(latitude, longitude).await {
                    Ok(address) => {
                        log::info!("Address: {address}");
                        state_clone.borrow_mut().address = address;
                    }
                    Err(err) => {
                        log::error!("Reverse geocode failed: {err}");
                        // We still have coordinates and can search with them; we
                        // just couldn't turn them into a readable address.
                        state_clone.borrow_mut().address =
                            "Found your location, but couldn't look up its address."
                                .to_string();
                    }
                }
                ctx_clone.request_repaint();
            });
        })
    }

    /*
    Builds the callback fired when the browser fails to get a position, mapping
    each PositionError code to a specific, actionable message.
     */
    #[cfg(target_arch = "wasm32")]
    fn make_error_callback(state: Rc<RefCell<LocationState>>,ctx: egui::Context) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PositionError)> {
        use wasm_bindgen::closure::Closure;
 
        Closure::<dyn FnMut(web_sys::PositionError)>::new(move |err: web_sys::PositionError| {
            let message = match err.code() {
                web_sys::PositionError::PERMISSION_DENIED => {
                    "Location access was denied. Allow it in your browser's site \
                    settings, or enter your address below instead."
                }
                web_sys::PositionError::POSITION_UNAVAILABLE => {
                    "Your location couldn't be determined right now. Try again, \
                    or enter your address below instead."
                }
                web_sys::PositionError::TIMEOUT => {
                    "Finding your location took too long. Try again, or enter \
                    your address below instead."
                }
                _ => "Couldn't get your location. Please enter your address below instead.",
            };
            log::error!("Geolocation error ({}): {}", err.code(), err.message());
 
            state.borrow_mut().status = LocationStatus::Error(message.to_string());
            ctx.request_repaint();
        })
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
    async fn geocode_suggestions(query: &str) -> Result<Vec<AddressSuggestion>, reqwest::Error> {
        let encoded_query = encode(query);
 
        // countrycodes=nz narrows results to New Zealand
        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2&limit=5&countrycodes=nz",encoded_query
        );
 
        let response = reqwest::Client::new().get(url).header("User-Agent", "ShopWise").send().await?;
 
        let results: Vec<GeocodeResponse> = response.json().await?;
 
        Ok(results
            .into_iter()
            .filter_map(|r| {
                let lat = r.lat.parse::<f64>().ok()?;
                let lon = r.lon.parse::<f64>().ok()?;
                Some(AddressSuggestion {
                    display_name: r.display_name,
                    lat,
                    lon,
                })
            }).collect())
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
