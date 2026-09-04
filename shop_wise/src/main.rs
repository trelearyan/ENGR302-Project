use std::fs::read_to_string;

use util::store::Store;
use crate::route_planner::filters::{StoreFilters, filter_stores};

pub mod input_gui;
pub mod output;
pub mod price_calculator;
pub mod results;
pub mod route_planner;
pub mod csv;
pub mod file_dialog;


use eframe::egui;
use input_gui::MyApp;


fn demo_route_planner() {
    let filters: StoreFilters =
        serde_json::from_str(read_to_string("filters.json").unwrap().as_str()).unwrap();

    let stores: Vec<Store> =
        serde_json::from_str(read_to_string("stores.json").unwrap().as_str()).unwrap();

    let output = filter_stores(&stores, &filters);

    print_bar();
    println!("List of all stores:");
    println!("{}", read_to_string("stores.json").unwrap().as_str());

    print_bar();
    println!("List of current user filters:");
    println!("{}", read_to_string("filters.json").unwrap().as_str());

    print_bar();
    println!("Filtered stores:");
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    print_bar();
}

fn print_bar() {
    let terminal_cols = termsize::get().unwrap().cols;
    for _ in 0..terminal_cols {
        print!("=");
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.check_file_results();
        egui::Panel::left("left_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.shopping_list_ui(ui);
                    ui.separator();
                    // fr 10
                    self.filters(ui);
                    ui.separator();
                    self.mileage(ui);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.location(ui);
            self.search_button(ui);
            // for debugging
            self.print_filters(ui);
            //fr09
            ui.separator();
            self.output_panel(ui);
        });
    }
}

// desktop app
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ShopWise",
        options,
        Box::new(|_| Ok(Box::new(MyApp::default()))),
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
                Box::new(|_| Ok(Box::new(MyApp::default()))),
            )
            .await
            .unwrap();

        document.get_element_by_id("loading_text").unwrap().remove();
    });
}
