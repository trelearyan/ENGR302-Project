pub const MIN_QUERY_LEN: usize = 3;

use crate::gui::LocationStatus;
use crate::gui::location_search::AddressSuggestion;
use eframe::egui::Context;
use serde::Deserialize;
use std::{cell::RefCell, rc::Rc};
use urlencoding::encode;

use crate::gui::location_search::LocationData;
#[derive(Deserialize)]
pub struct GeocodeResponse {
    lat: String,
    lon: String,
    display_name: String,
}

// store readable address (after converting from coords to address)
#[derive(Deserialize)]
pub struct ReverseResponse {
    display_name: String,
}
pub fn get_current_location(state: Rc<RefCell<LocationData>>, ctx: Context) {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let navigator = window.navigator();

    let geolocation = match resolve_geolocation(&navigator, &state, &ctx) {
        Some(g) => g,
        None => return,
    };

    // egui only repaints in response to input events by default without this, a state change made from a background/async callback might not actually appear on screen until the user happens to move the mouse.
    state.borrow_mut().status = LocationStatus::Locating;
    ctx.request_repaint();

    // Options control "browser hangs forever" failure mode:
    // - timeout: give up after 10s instead of waiting indefinitely
    // - maximum_age: accept a cached fix up to 60s old so a repeat click returns near-instantly
    let options = web_sys::PositionOptions::new();
    options.set_timeout(10_000);
    options.set_maximum_age(60_000);

    let success = make_success_callback(state.clone(), ctx.clone());
    let error = make_error_callback(state.clone(), ctx.clone());

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
pub fn resolve_geolocation(
    navigator: &web_sys::Navigator,
    state: &Rc<RefCell<LocationData>>,
    ctx: &Context,
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
pub fn make_success_callback(
    state: Rc<RefCell<LocationData>>,
    ctx: Context,
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
            match reverse_geocode(latitude, longitude).await {
                Ok(address) => {
                    log::info!("Address: {address}");
                    state.borrow_mut().address = address;
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
pub fn make_error_callback(
    state: Rc<RefCell<LocationData>>,
    ctx: Context,
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
pub async fn reverse_geocode(latitude: f64, longitude: f64) -> Result<String, reqwest::Error> {
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

pub async fn geocode_suggestions(query: &str) -> Result<Vec<AddressSuggestion>, reqwest::Error> {
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
