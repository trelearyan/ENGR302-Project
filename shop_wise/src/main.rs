
use crate::{
    gui::ShowableWidget,
    route_planner::filters::{StoreFilters, filter_stores},
};
use util::store::Store;

pub mod filehandling;

pub mod gui;
pub mod price_calculator;
pub mod route_planner;

use eframe::egui;
use gui::AppData;

impl eframe::App for AppData {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

// desktop app
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ShopWise",
        options,
        Box::new(|_| Ok(Box::new(AppData::default()))),
    )
}

// website
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen_futures::spawn_local;
    use web_sys::wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(AppData::default()))),
            )
            .await
            .unwrap();

        document.get_element_by_id("loading_text").unwrap().remove();
    });
}
