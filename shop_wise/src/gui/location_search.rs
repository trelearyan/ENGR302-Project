use eframe::egui::{Color32, Frame, Key, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use util::coordinate::Coordinate;

use crate::gui::ShowableWidget;

const DEBOUNCE_SECONDS: f64 = 0.4;
const MIN_QUERY_LEN: usize = 3;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct LocationData {
    pub latitude: Option<f64>,  // TODO: this should be one Option<Coordinate>
    pub longitude: Option<f64>, // , not possible to have one but not the other

    pub address: String,
    pub status: LocationStatus,
    pub suggestions: Vec<AddressSuggestion>,

    pub request_id: u64,
    pub last_edit_time: Option<f64>,
}

// A single resolved candidate shown in the suggestions dropdown.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AddressSuggestion {
    display_name: String,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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

impl From<LocationData> for Coordinate {
    fn from(value: LocationData) -> Self {
        if let (Some(latitude), Some(longitude)) = (value.latitude, value.longitude) {
            Coordinate::from_lat_long_f64(latitude, longitude)
        } else {
            Coordinate::default()
        }
    }
}
impl ShowableWidget for LocationData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Location");

        if ui.button("Use Current Location").clicked() {
            log::info!("Requesting location...");
            #[cfg(target_arch = "wasm32")]
            self.get_current_location(self.location_state.clone(), ui.ctx().clone());
        }

        let now = ui.input(|i| i.time);

        let response = ui.add(
            TextEdit::singleline(&mut self.address)
                .hint_text("e.g. 12 Example Street, Suburb, City"),
        );

        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));

        if response.changed() {
            self.status = LocationStatus::Idle;
            self.suggestions.clear();
            self.latitude = None;
            self.longitude = None;
            self.last_edit_time = Some(now);
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(DEBOUNCE_SECONDS));
        }

        // on Enter, Fire a search once the debounce window has elapsed, or immediately
        let should_fire = match self.last_edit_time {
            Some(_last_edit) if enter_pressed => true,
            Some(last_edit) if now - last_edit >= DEBOUNCE_SECONDS => true,
            _ => false,
        };

        if should_fire {
            let query = self.address.trim().to_string();
            self.last_edit_time = None;

            if query.len() < MIN_QUERY_LEN {
                self.status = LocationStatus::Idle;
            } else {
                self.status = LocationStatus::Searching;
                self.request_id += 1;
                let _this_request = self.request_id;
                let _state_clone = self.clone();
                let _ctx_clone = ui.ctx().clone();
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen_futures::spawn_local;
                    spawn_local(async move {
                        match AppData::geocode_suggestions(&query).await {
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

        match &self.status {
            LocationStatus::Searching => {
                ui.label("Searching…");
            }
            LocationStatus::NotFound => {
                ui.colored_label(
                    Color32::from_rgb(200, 60, 60),
                    "No matching address found — try adding your suburb or city.",
                );
            }
            LocationStatus::Resolved | LocationStatus::Success => {
                ui.colored_label(Color32::from_rgb(60, 160, 60), "Location set");
            }
            LocationStatus::Locating => {
                ui.label("Finding your current location…");
            }
            LocationStatus::Error(message) => {
                ui.colored_label(Color32::from_rgb(200, 60, 60), message.clone());
            }
            LocationStatus::Idle | LocationStatus::Suggesting => {}
        }

        // suggestions dropdown
        if self.status == LocationStatus::Suggesting {
            let suggestions = self.suggestions.clone();
            Frame::popup(ui.style()).show(ui, |ui| {
                for s in &suggestions {
                    if ui.selectable_label(false, &s.display_name).clicked() {
                        self.address = s.display_name.clone();
                        self.latitude = Some(s.lat);
                        self.longitude = Some(s.lon);
                        self.status = LocationStatus::Resolved;
                        self.suggestions.clear();
                        self.last_edit_time = None;
                    }
                }
            });
        }
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
fn resolve_geolocation(
    navigator: &web_sys::Navigator,
    state: &Rc<RefCell<LocationState>>,
    ctx: &egui::Context,
) -> Option<web_sys::Geolocation> {
    match navigator.geolocation() {
        Ok(g) => Some(g),
        Err(_) => {
            state.borrow_mut().status = LocationStatus::Error(
                "Geolocation isn't available. This usually means the page \
                     isn't served over HTTPS, or your browser doesn't support it."
                    .to_string(),
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
fn make_success_callback(
    state: Rc<RefCell<LocationState>>,
    ctx: egui::Context,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Position)> {
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
            match AppData::reverse_geocode(latitude, longitude).await {
                Ok(address) => {
                    log::info!("Address: {address}");
                    state_clone.borrow_mut().address = address;
                }
                Err(err) => {
                    log::error!("Reverse geocode failed: {err}");
                    // We still have coordinates and can search with them; we
                    // just couldn't turn them into a readable address.
                    state_clone.borrow_mut().address =
                        "Found your location, but couldn't look up its address.".to_string();
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
fn make_error_callback(
    state: Rc<RefCell<LocationState>>,
    ctx: egui::Context,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PositionError)> {
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
        "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2&limit=5&countrycodes=nz",
        encoded_query
    );

    let response = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "ShopWise")
        .send()
        .await?;

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
        })
        .collect())
}


