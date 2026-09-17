use std::rc::Rc;

use eframe::egui::{self, Ui};
use serde::ser::SerializeStruct;
use util::search::SearchUnits::{DOLLAR, EACH, KILOGRAM, GRAM, LITRE, MILLILITRE};
use util::search::{ShoppingItemQuery};

use crate::filehandling::file_dialog::{FileChannel, FileOutcome};
use crate::gui::ShowableWidget;

#[derive(Clone, Debug, Default)]
pub struct ShoppingListData {
    shopping_items: Vec<ShoppingItemQuery>,
    files: Rc<FileChannel>,
    csv_status: Option<String>,
    cleared_items: Option<Vec<ShoppingItemQuery>>,
}

impl Serialize for ShoppingListData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut to_serial = serializer.serialize_struct("ShoppingListData", 3)?;
        to_serial.serialize_field("shopping_items", &self.shopping_items)?;
        to_serial.serialize_field("csv_status", &self.shopping_items)?;
        to_serial.serialize_field("cleared_items", &self.shopping_items)?;
        to_serial.end()
    }
}

impl ShowableWidget for ShoppingListData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Your shopping list");

        ui.horizontal(|ui| {
            if ui.button("+ Add Item").clicked() {
                self.shopping_items.push(ShoppingItemQuery {
                    name: String::new(),
                    quantity: 1,
                    unit: EACH.to_str().to_string(),
                });
                self.cleared_items = None;
            }

            if ui.button("Load CSV").clicked() {
                self.load_csv();
            }

            let can_save = self
                .shopping_items
                .iter()
                .any(|item| !item.name.trim().is_empty());

            if ui
                .add_enabled(can_save, egui::Button::new("Save CSV"))
                .clicked()
            {
                self.save_csv();
            }

            let has_items = !self.shopping_items.is_empty();
            if ui
                .add_enabled(has_items, egui::Button::new("Clear"))
                .on_hover_text("Remove all items")
                .clicked()
            {
                self.cleared_items = Some(std::mem::take(&mut self.shopping_items));
                self.csv_status = None;
            }
        });

        if let Some(cleared) = &self.cleared_items {
            let count = cleared.len();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Removed {count} items"))
                        .small()
                        .weak(),
                );
                if ui.button("Undo").clicked()
                    && let Some(items) = self.cleared_items.take()
                {
                    self.shopping_items = items;
                }
            });
        }

        if let Some(status) = &self.csv_status {
            ui.label(egui::RichText::new(status).small().weak());
        }
        let mut remove_index: Option<usize> = None;

        for (i, item) in self.shopping_items.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut item.name);
                ui.add(egui::DragValue::new(&mut item.quantity));

                egui::ComboBox::from_id_salt(i)
                    .selected_text(&item.unit)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut item.unit,
                            EACH.to_str().to_string(),
                            EACH.to_str(),
                        );
                        ui.selectable_value(
                            &mut item.unit,
                            GRAM.to_str().to_string(),
                            GRAM.to_str(),
                        );
                        ui.selectable_value(
                            &mut item.unit,
                            KILOGRAM.to_str().to_string(),
                            KILOGRAM.to_str(),
                        );
                        ui.selectable_value(
                            &mut item.unit,
                            MILLILITRE.to_str().to_string(),
                            MILLILITRE.to_str(),
                        );
                        ui.selectable_value(
                            &mut item.unit,
                            LITRE.to_str().to_string(),
                            LITRE.to_str(),
                        );
                        ui.selectable_value(
                            &mut item.unit,
                            DOLLAR.to_str().to_string(),
                            DOLLAR.to_str(),
                        );
                    });

                if ui
                    .add(egui::Button::new("X").fill(egui::Color32::RED))
                    .on_hover_text("Remove item")
                    .clicked()
                {
                    remove_index = Some(i);
                }
            });
        }

        if let Some(i) = remove_index {
            self.shopping_items.remove(i);
        }
    }
}

impl ShoppingListData {
    fn load_csv(&mut self) {
        self.files.open("CSV", &["csv"]);
    }

    fn save_csv(&mut self) {
        let text = crate::filehandling::csv::write_to_csv(
            self.shopping_items
                .iter()
                .filter(|item| !item.name.trim().is_empty()),
        );
        //let text = crate::csv::write_to_csv(&self.shopping_items);
        self.files.save(
            text,
            "Shopwise shopping list.csv".to_owned(),
            "CSV",
            &["csv"],
        );
    }

    fn check_file_results(&mut self) {
        while let Some(outcome) = self.files.poll() {
            self.csv_status = Some(match outcome {
                FileOutcome::Opened { text, name } => {
                    match crate::filehandling::csv::read_csv(&text) {
                        Ok(items) => {
                            let _count = items.len();
                            self.shopping_items = items;
                            //self.undo = None;
                            format!("Loaded {name}")
                        }
                        Err(error) => format!("could not read {name}: {error}"),
                    }
                }
                FileOutcome::Saved { name } => format!("Saved to {name}"),
                FileOutcome::Failed(reason) => reason,
            });
        }
    }
}
