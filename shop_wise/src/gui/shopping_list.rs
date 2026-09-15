use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{self, Ui};
use serde::Serialize;
use serde::ser::SerializeStruct;
use util::search::ShoppingItemQuery;

use crate::filehandling::file_dialog::FileChannel;
use crate::gui::ShowableWidget;

const DEBOUNCE_SECONDS: f64 = 0.4;
const MIN_QUERY_LEN: usize = 3;
 
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum AddItemStatus {
    #[default]
    Idle,
    Searching,
    Suggesting,
    NotFound,
}
 
#[derive(Debug, Default, Clone)]
struct AddItemState {
    query: String,
    status: AddItemStatus,
    suggestions: Vec<ShoppingItemQuery>,
    request_id: u64,
    last_edit_time: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct ShoppingListData {
    pub shopping_items: Vec<ShoppingItemQuery>,
    pub files: Rc<FileChannel>,
    pub csv_status: Option<String>,
    pub cleared_items: Option<Vec<ShoppingItemQuery>>,
    add_item_modal: Option<Rc<RefCell<AddItemState>>>,
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
        self.check_file_results();
        ui.heading("Your shopping list");

        ui.horizontal(|ui| {
            if ui.button("+ Add Item")
                .on_hover_text("Add an item to your shopping list")
                .clicked() {
                self.add_item_modal = Some(Rc::new(RefCell::new(AddItemState::default())));
                self.cleared_items = None;
            }

            if ui.button("Load CSV")
                .on_hover_text("Load in your own shopping list CSV file")
                .clicked() {
                self.load_csv();
            }

            let can_save = self
                .shopping_items
                .iter()
                .any(|item| !item.name.trim().is_empty());

            if ui
                .add_enabled(can_save, egui::Button::new("Save CSV"))
                .on_hover_text("Save your shopping list as a CSV file")
                .on_disabled_hover_text("Add items to your Shopping List to save")
                .clicked()
            {
                self.save_csv();
            }

            let has_items = !self.shopping_items.is_empty();
            if ui
                .add_enabled(has_items, egui::Button::new("Clear"))
                .on_hover_text("Remove all items from your Shopping List")
                .on_disabled_hover_text("Add items to your Shopping List use clear button")
                .clicked()
            {
                self.cleared_items = Some(std::mem::take(&mut self.shopping_items));
                self.csv_status = None;
            }
        });

        self.show_add_item_search(ui);

        if let Some(cleared) = &self.cleared_items {
            let count = cleared.len();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Removed {count} items"))
                        .small()
                        .weak(),
                );
                if ui.button("Undo")
                    .on_hover_text("Return your cleared items back to your shopping list")
                    .clicked()
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
        // Items are now resolved+locked; no more free-text editing here.
        for (i, item) in self.shopping_items.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("X").fill(egui::Color32::RED))
                    .on_hover_text("Remove this item from your shopping list")
                    .clicked()
                {
                    remove_index = Some(i);
                }
                ui.label(&item.name);
                ui.label(format!("{} {}", item.quantity, item.unit));
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

    pub fn check_file_results(&mut self) {
        use crate::filehandling::file_dialog::FileOutcome;

        while let Some(outcome) = self.files.poll() {
            self.csv_status = Some(match outcome {
                FileOutcome::Opened { text, name } => {
                    match crate::filehandling::csv::read_csv(&text) {
                        Ok(items) => {
                            let _count = items.len();
                            self.shopping_items = items;
                            self.cleared_items = None;
                            format!("Loaded {name}")
                        }
                        Err(error) => format!("Could not load {name}: {error}"),
                    }
                }
                FileOutcome::Saved { name } => format!("Saved to {name}"),
                FileOutcome::Failed(reason) => reason,
            });
        }
    }

    fn show_add_item_search(&mut self, ui: &mut Ui) {
        let Some(state_rc) = self.add_item_modal.clone() else {
            return;
        };
        let now = ui.input(|i| i.time);
        let mut close = false;
        let mut picked: Option<ShoppingItemQuery> = None;
 
        {
            let mut state = state_rc.borrow_mut();
 
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.query)
                        .hint_text("Enter Your Item Name (e.g. milk)"),
                );
 
                if response.changed() {
                    state.status = AddItemStatus::Idle;
                    state.suggestions.clear();
                    state.last_edit_time = Some(now);
                    ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(
                        DEBOUNCE_SECONDS,
                    ));
                }
 
                if ui.button("Finish!").clicked() {
                    close = true;
                }
            });
 
            let should_fire = matches!(
                state.last_edit_time,
                Some(last_edit) if now - last_edit >= DEBOUNCE_SECONDS
            );
 
            if should_fire {
                let query = state.query.trim().to_string();
                state.last_edit_time = None;
 
                if query.len() < MIN_QUERY_LEN {
                    state.status = AddItemStatus::Idle;
                } else {
                    state.status = AddItemStatus::Searching;
                    state.request_id += 1;
 
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen_futures::spawn_local;
 
                        let this_request = state.request_id;
                        let state_clone = state_rc.clone();
                        let ctx_clone = ui.ctx().clone();
 
                        spawn_local(async move {
                            let result = ShoppingListData::mock_search_products(&query).await;
                            let mut state = state_clone.borrow_mut();
                            if state.request_id == this_request {
                                match result {
                                    Ok(results) if !results.is_empty() => {
                                        state.suggestions = results;
                                        state.status = AddItemStatus::Suggesting;
                                    }
                                    _ => state.status = AddItemStatus::NotFound,
                                }
                            }
                            ctx_clone.request_repaint();
                        });
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Native search isn't implemented, surface as not found rather than leaving status stuck on Searching (prevent CI ensure no warnings)
                        state.status = AddItemStatus::NotFound;
                    }
                }
            }
 
            match state.status {
                AddItemStatus::Searching => {
                    ui.label("Searching…");
                }
                AddItemStatus::NotFound => {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 60, 60),
                        "No matching products found.",
                    );
                }
                AddItemStatus::Idle | AddItemStatus::Suggesting => {}
            }
 
            if state.status == AddItemStatus::Suggesting {
                let suggestions = state.suggestions.clone();
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    for s in &suggestions {
                        let label = format!("{}  ·  {} {}", s.name, s.quantity, s.unit);
                        if ui.selectable_label(false, label).clicked() {
                            picked = Some(s.clone());
                        }
                    }
                });
            }
        } // `state` (the RefMut borrow) drops here, before touching self.* below
 
        if let Some(s) = picked {
            self.shopping_items.push(s);
            self.cleared_items = None;
            let mut state = state_rc.borrow_mut();
            state.query.clear();
            state.suggestions.clear();
            state.status = AddItemStatus::Idle;
        }
 
        if close {
            self.add_item_modal = None;
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn mock_search_products(query: &str) -> Result<Vec<ShoppingItemQuery>, String> {
        // MOCK ONLY
        let base = query.trim();
        if base.is_empty() {
            return Ok(Vec::new());
        }
 
        let units = ["each", "L", "kg", "g"];
 
        let suggestions = (0..10u8)
            .map(|i| {
                let suffix: String = if i == 0 {
                    String::new()
                } else {
                    (b'b'..=b'b' + i - 1).map(|c| c as char).collect()
                };
                ShoppingItemQuery {
                    name: format!("{base}{suffix}"),
                    quantity: 1,
                    unit: units[i as usize % units.len()].to_string(),
                }
            })
            .collect();
 
        Ok(suggestions)
    }
}

