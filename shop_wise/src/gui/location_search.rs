use eframe::egui::{Color32, Frame, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use util::coordinate::Coordinate;

use crate::gui::ShowableWidget;

#[cfg(target_arch = "wasm32")]
pub mod evil_wasm;

const DEBOUNCE_SECONDS: f64 = 0.4;

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

impl ShowableWidget for LocationData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Location");

        if ui.button("Use Current Location").clicked() {
            log::info!("Requesting location...");

            #[cfg(target_arch = "wasm32")]
            {
                use std::{cell::RefCell, rc::Rc};

                evil_wasm::get_current_location(
                    Rc::new(RefCell::new(self.clone())),
                    ui.ctx().clone(),
                );
            }
        }

        let now = ui.input(|i| i.time);

        let response = ui.add(
            TextEdit::singleline(&mut self.address)
                .hint_text("e.g. 12 Example Street, Suburb, City"),
        );

        if response.changed() {
            self.status = LocationStatus::Idle;
            self.suggestions.clear();
            self.latitude = None;
            self.longitude = None;
            self.last_edit_time = Some(now);
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(DEBOUNCE_SECONDS));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use eframe::egui::Key;
            use std::borrow::BorrowMut;
            use std::{cell::RefCell, rc::Rc};

            let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));

            // on Enter, Fire a search once the debounce window has elapsed, or immediately
            let should_fire = match self.last_edit_time {
                Some(_last_edit) if enter_pressed => true,
                Some(last_edit) if now - last_edit >= DEBOUNCE_SECONDS => true,
                _ => false,
            };

            if should_fire {
                let query = self.address.trim().to_string();
                self.last_edit_time = None;

                if query.len() < crate::gui::location_search::evil_wasm::MIN_QUERY_LEN {
                    self.status = LocationStatus::Idle;
                } else {
                    self.status = LocationStatus::Searching;
                    self.request_id += 1;
                    let this_request = self.request_id;
                    let mut state_clone = self.clone();
                    let ctx_clone = ui.ctx().clone();
                    {
                        use wasm_bindgen_futures::spawn_local;
                        spawn_local(async move {
                            match evil_wasm::geocode_suggestions(&query).await {
                                Ok(results) => {
                                    let state = state_clone.borrow_mut();
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
                                    log::error!("Geocode search failed: {err}");
                                    let state = state_clone.borrow_mut();
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
                for s in suggestions {
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

/*
Getting the user location
*/
// TODO: unused?
// A single resolved candidate shown in the suggestions dropdown.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AddressSuggestion {
    display_name: String,
    lat: f64,
    lon: f64,
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
