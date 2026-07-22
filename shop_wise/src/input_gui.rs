use eframe::egui;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use urlencoding::encode;

#[derive(Default)]
struct LocationState {
    latitude: Option<f64>,
    longitude: Option<f64>,
    // location (what the user sees)
    address: String,
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

pub struct MyApp {
    max_range: u32,
    max_stores: u32,
    include_paknsave: bool,
    include_newworld: bool,
    include_woolies: bool,

    // Shared location state
    location_state: Rc<RefCell<LocationState>>,
}

impl MyApp {
    fn new(
        max_range: u32,
        max_stores: u32,
        include_paknsave: bool,
        include_newworld: bool,
        include_woolies: bool,
    ) -> Self {
        Self {
            max_range,
            max_stores,
            include_paknsave,
            include_newworld,
            include_woolies,

            location_state: Rc::new(
                RefCell::new(LocationState::default())
            )
        }
    }
    // for debugging
    pub fn print_filters(&self, ui: &mut egui::Ui) {
        ui.label("=== Filter values ===");
        ui.label(format!("Max Range: {}", self.max_range));
        ui.label(format!("Max Stores: {}", self.max_stores));
        ui.label(format!("Pak'nSave: {}", self.include_paknsave));
        ui.label(format!("New World: {}", self.include_newworld));
        ui.label(format!("Woolworths: {}", self.include_woolies));
    }

    pub fn filters(&mut self, ui: &mut egui::Ui){
        ui.heading("Filters");

        ui.label("Max Range");
        ui.add(egui::Slider::new(&mut self.max_range, 1..=50).text("(km)"),);

        ui.label("Max Stores per Trip");
        ui.add(egui::Slider::new(&mut self.max_stores, 1..=10));

        ui.separator();

        ui.label("Supermarkets");
        ui.checkbox(&mut self.include_paknsave, "Pak'nSave");

        ui.checkbox(&mut self.include_newworld, "New World");

        ui.checkbox(&mut self.include_woolies, "Woolworths");
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
            ui.add(egui::TextEdit::singleline(&mut state.address).hint_text("Enter an address"),);

            let response = ui.add_enabled(!state.address.trim().is_empty(), egui::Button::new("Find Location"));

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
                                log::error!(
                                    "Failed to geocode address: {}",err
                                );
                            }
                        }
                    });
                }
            }
        });
    }

    pub fn search_button(&self, ui: &mut egui::Ui){
        ui.horizontal(|ui|{
            if ui.button("Search").clicked(){
                log::info!("the search button is clicked!");
                // First: check if both shopping list and location are not empty
        
                // Second: API Call e.g. item_resolver(self.shopping_items) <Sam>

                // Third: API Call e.g. route_planner(Location, Filters) <Alex>
            }
        }); 
    }

    /*
    Getting the user location
     */
    #[cfg(target_arch = "wasm32")]
    fn get_current_location(&self, state: Rc<RefCell<LocationState>>) {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        // get browser window
        let window = web_sys::window().unwrap();

        let navigator = window.navigator();

        let geolocation = navigator.geolocation().unwrap();

        // call back
        let success = Closure::<dyn FnMut(web_sys::Position)>::new(
            move |position: web_sys::Position| {
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
                log::info!("Latitude: {}, Longitude: {}",latitude,longitude); // debug
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
            },
        );

        geolocation.get_current_position(success.as_ref().unchecked_ref()).unwrap();
        success.forget(); // keep this callback alive
    }

    // translate coords --> readable address
    #[cfg(target_arch = "wasm32")]
    async fn reverse_geocode(latitude: f64, longitude: f64) -> Result<String, reqwest::Error> {

        let url = format!("https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={}&lon={}",latitude,longitude);

        let response = reqwest::Client::new().get(url).header("User-Agent","ShopWise").send().await?;

        let result: ReverseResponse =response.json().await?;

        Ok(result.display_name)
    }

    #[cfg(target_arch = "wasm32")]
    async fn geocode(address: &str) -> Result<(f64, f64, String), reqwest::Error> {
        let encoded_address = encode(address);

        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2&limit=1",encoded_address
        );

        let response = reqwest::Client::new().get(url).header("User-Agent", "ShopWise").send().await?;

        let result: Vec<GeocodeResponse> = response.json().await?;

        if let Some(first) = result.first() {
            let latitude = first.lat.parse::<f64>().unwrap();
            let longitude = first.lon.parse::<f64>().unwrap();

            Ok((
                latitude,
                longitude,
                first.display_name.clone(),
            ))
        } else {
            log::error!("Location not found.");

            Ok((0.0, 0.0, "Location not found".to_string()))
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new(
            10,
            3,
            true,
            true,
            true,
        )
    }
}