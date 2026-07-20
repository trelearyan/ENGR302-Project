mod input_gui;
use input_gui::MyApp;
use eframe::egui;

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("left_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui|{
                    // fr 10
                    self.filters(ui);
                }); 
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // for debugging
            self.print_filters(ui);
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
        let document = web_sys::window()
            .unwrap()
            .document()
            .unwrap();

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

        document
            .get_element_by_id("loading_text")
            .unwrap()
            .remove();
    });
}