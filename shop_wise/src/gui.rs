#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
// use web_sys::wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
// creating gui application
pub fn show_gui() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "shopping",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

// When compiling to web using trunk:
//#[cfg(target_arch = "wasm32")]
// fn main() {
//     use web_sys::wasm_bindgen::JsCast as _;

//     // Redirect `log` message to `console.log` and friends:
//     eframe::WebLogger::init(log::LevelFilter::Debug).ok();

//     let web_options = eframe::WebOptions::default();

//     wasm_bindgen_futures::spawn_local(async {
//         let document = web_sys::window()
//             .expect("No window")
//             .document()
//             .expect("No document");

//         let canvas = document
//             .get_element_by_id("the_canvas_id")
//             .expect("Failed to find the_canvas_id")
//             .dyn_into::<web_sys::HtmlCanvasElement>()
//             .expect("the_canvas_id was not a HtmlCanvasElement");

//         let start_result = eframe::WebRunner::new()
//             .start(
//                 canvas,
//                 web_options,
//                 Box::new(|cc| Ok(Box::new(<MyApp>::default()))),
//             )
//             .await;

//         // Remove the loading text and spinner:
//         if let Some(loading_text) = document.get_element_by_id("loading_text") {
//             match start_result {
//                 Ok(_) => {
//                     loading_text.remove();
//                 }
//                 Err(e) => {
//                     loading_text.set_inner_html(
//                         "<p> The app has crashed. See the developer console for details. </p>",
//                     );
//                     panic!("Failed to start eframe: {e:?}");
//                 }
//             }
//         }
//     });
// }

// similar idea to java class
struct MyApp {
    shopping_list: String,
    // u32 means int
    max_range: u32,
    max_stores: u32,
    include_paknsave: bool,
    include_newworld: bool,
    include_woolies: bool,
}

impl MyApp { // methods/functions for MyApp (like methods inside a java class)
    fn new( // constructor
        shopping_list: String,
        max_range: u32,
        max_stores: u32,
        include_paknsave: bool,
        include_newworld: bool,
        include_woolies: bool,
    ) -> Self {
        Self { // create a MyApp object
            shopping_list,
            max_range,
            max_stores,
            include_paknsave,
            include_newworld,
            include_woolies,
        }
    }

    // Prints the current filter state.
    // Called whenever any filter value changes.
    fn print_filters(&self) {
        println!("=== Filter values passed to Route Planner ===");
        println!("  Max range:       {} km", self.max_range);
        println!("  Max stores:      {}", self.max_stores);
        println!("  Pak'n'Save:      {}", self.include_paknsave);
        println!("  New World:       {}", self.include_newworld);
        println!("  Woolworths:      {}", self.include_woolies);
        println!("=============================================");
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new(" - \n".to_string().repeat(5), 10, 3, true, true, true)
    }
}


impl eframe::App for MyApp {
    // this function gets called repeatedly while the app runs.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

        egui::Panel::left("left")
            .resizable(true)
            .show_inside(ui, |ui| {

                ui.heading("Your shopping list");

                ui.text_edit_multiline(&mut self.shopping_list);

                ui.separator();

                ui.heading("Filters");

                // Max range slider
                ui.label("Max Range");
                let range_changed = ui
                    .add(
                        egui::Slider::new(&mut self.max_range, 1..=50)
                            .text("(km)")
                    )
                    .changed();

                // Max stores slider
                ui.label("Max Stores per Trip");
                let stores_changed = ui
                    .add(
                        egui::Slider::new(&mut self.max_stores, 1..=10)
                    )
                    .changed();

                ui.label("Supermarkets");

                // Supermarket checkboxes
                let paknsave_changed = ui
                    .checkbox(&mut self.include_paknsave, "Pak'n'Save")
                    .changed();

                let newworld_changed = ui
                    .checkbox(&mut self.include_newworld, "New World")
                    .changed();

                let woolies_changed = ui
                    .checkbox(&mut self.include_woolies, "Woolworths")
                    .changed();

                // Print filters if ANY value changed
                if range_changed
                    || stores_changed
                    || paknsave_changed
                    || newworld_changed
                    || woolies_changed
                {
                    self.print_filters();
                }
            });

        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                ui.heading("");
            });
    }
}