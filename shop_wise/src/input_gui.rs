use eframe::egui;
//use serde::Deserialize;
//use std::cell::RefCell;
//use std::rc::Rc;

pub struct MyApp {
    max_range: u32,
    max_stores: u32,
    include_paknsave: bool,
    include_newworld: bool,
    include_woolies: bool,
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