use std::str::FromStr;

use bigdecimal::BigDecimal;
use derive_more::{Display, IsVariant};
use eframe::egui::{self, Ui};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use util::{cost::Cost, speed::Speed};

use crate::gui::ShowableWidget;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct TransitData {
    pub mileage_option: MileageOptions,
    pub mileage_scratch: String,
}

impl ShowableWidget for TransitData {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Transit")
            .on_hover_text("What will be your travel costs for your shop");
        ui.horizontal(|ui| {
            ui.label("$");
            ui.add_enabled_ui(self.mileage_option.is_custom(), |ui| {
                let field = ui
                    .add_sized(
                        [40.0, 20.0],
                        egui::TextEdit::singleline(&mut self.mileage_scratch),
                    )
                    .on_hover_text("Enable \"Custom\" to the right to enter a specific value");

                if field.lost_focus() {
                    if let Ok(new_value) =
                        BigDecimal::from_str(self.mileage_scratch.as_str()).map(Cost::new)
                    {
                        self.mileage_option = new_value.into();
                    } else {
                        self.mileage_scratch = Cost::from(self.mileage_option.clone()).to_string();
                    }
                } else if !field.has_focus() {
                    self.mileage_scratch = Cost::from(self.mileage_option.clone()).to_string();
                }
            });
            ui.label("/km")
                .on_hover_text("Cost per kilometre travelled");
            egui::ComboBox::from_id_salt("the combobox to select mileage option")
                .width(80.0)
                .truncate()
                .selected_text(self.mileage_option.to_string() + "         ") // spaces needed to have constant size box
                .show_ui(ui, |ui| {
                    for option in MileageOptions::iter() {
                        ui.selectable_value(
                            &mut self.mileage_option,
                            option.clone(),
                            option.to_string().as_str(),
                        );
                    }
                })
                .response
                .on_hover_text("Select your vehicle type or create a custom rate.");
        });
    }
}

// https://www.ird.govt.nz/income-tax/income-tax-for-businesses-and-organisations/types-of-business-expenses/claiming-vehicle-expenses/kilometre-rates-2025-2026
#[derive(
    Debug, Display, EnumIter, IsVariant, Clone, PartialEq, Default, Serialize, Deserialize,
)]
pub enum MileageOptions {
    #[default]
    Petrol,
    Diesel,
    Hybrid,
    Electric,
    #[display("Custom mileage")]
    Custom(Cost),
    #[display("Dont calculate mileage")]
    DontCalculateMileage,
}

impl From<MileageOptions> for Cost {
    fn from(value: MileageOptions) -> Self {
        match value {
            MileageOptions::Petrol => Cost::from_cents(37),
            MileageOptions::Diesel => Cost::from_cents(38),
            MileageOptions::Hybrid => Cost::from_cents(24),
            MileageOptions::Electric => Cost::from_cents(23),
            MileageOptions::Custom(cost) => cost,
            MileageOptions::DontCalculateMileage => Cost::from_cents(0),
        }
    }
}

impl From<Cost> for MileageOptions {
    fn from(value: Cost) -> Self {
        match value {
            x if x == Cost::from_cents(37) => MileageOptions::Petrol,
            x if x == Cost::from_cents(38) => MileageOptions::Diesel,
            x if x == Cost::from_cents(24) => MileageOptions::Hybrid,
            x if x == Cost::from_cents(23) => MileageOptions::Electric,
            x if x == Cost::from_cents(0) => MileageOptions::DontCalculateMileage,
            custom => MileageOptions::Custom(custom),
        }
    }
}

impl MileageOptions {
    #[must_use]
    pub fn average_speed(&self) -> Speed {
        // Assuming slightly under standard city limit of 50 due to traffic,
        // starting and ending on a lower speed local road, etc.
        Speed::from_kilometres_per_hour(40)
    }
}
