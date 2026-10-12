use std::ops::Deref;
use eframe::egui;

use crate::{
    gui::ShowableWidget,
    price_calculator::results::{
        Results, ResultsState, ScenarioKind, StoreStop, UNAVAILABLE, UnresolvedItem, brand_label,
        format_delta, format_distance, format_duration, format_money,
    },
};

impl ShowableWidget for OutputRoutesData {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.showold(ui);
    }
}

const STACK_BELOW_WIDTH: f32 = 760.0;
const CARD_MIN_HEIGHT: f32 = 140.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PanelLayout {
    #[default]
    Cards,
    Tabs,
}

#[derive(Clone, Debug)]
pub struct OutputRoutesData {
    pub selected: ScenarioKind,
    pub layout: PanelLayout,
    pub results: ResultsState,
}

impl Default for OutputRoutesData {
    fn default() -> Self {
        Self {
            selected: ScenarioKind::Best,
            layout: PanelLayout::Cards,
            results: ResultsState::Idle,
        }
    }
}

impl OutputRoutesData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn selected(&self) -> ScenarioKind {
        self.selected
    }

    #[must_use]
    pub fn layout(&self) -> PanelLayout {
        self.layout
    }

    pub fn set_layout(&mut self, layout: PanelLayout) {
        self.layout = layout;
    }

    #[must_use]
    pub fn with_layout(mut self, layout: PanelLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn showold(&mut self, ui: &mut egui::Ui) {
        let weak_colour = ui.visuals().weak_text_color();
        let error_colour = ui.visuals().error_fg_color;

        match self.results.clone() {
                ResultsState::Idle => Self::message(
                    ui,
                    "Nothing to show yet",
                    "Add at least one item to your shopping list, set your location, then run a search.",
                    weak_colour,
                ),
                ResultsState::Loading => Self::loading(ui),
                ResultsState::Failed(reason) => Self::message(
                    ui,
                    "That search could not be completed",
                    reason.as_str(),
                    error_colour,
                ),
                ResultsState::Ready(results) => self.ready(ui, results.deref()),
        }
    }

    fn ready(&mut self, ui: &mut egui::Ui, results: &Results) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Starting from").small().weak());
            ui.label(
                egui::RichText::new(results.origin_label.as_str())
                    .small()
                    .strong(),
            );
        });
        ui.add_space(8.0);

        if !results.unresolved.is_empty() {
            Self::unresolved_banner(ui, &results.unresolved);
            ui.add_space(8.0);
        }

        match self.layout {
            PanelLayout::Cards => self.scenario_cards(ui, results),
            PanelLayout::Tabs => self.scenario_tabs(ui, results),
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        Self::plan_detail(ui, results, self.selected);
    }

    fn scenario_cards(&mut self, ui: &mut egui::Ui, results: &Results) {
        let selected = self.selected;
        let mut clicked: Option<ScenarioKind> = None;

        if ui.available_width() < STACK_BELOW_WIDTH {
            for kind in ScenarioKind::ALL {
                if Self::scenario_card(ui, results, kind, selected) {
                    clicked = Some(kind);
                }
                ui.add_space(8.0);
            }
        } else {
            ui.columns(ScenarioKind::ALL.len(), |columns| {
                for (column, kind) in columns.iter_mut().zip(ScenarioKind::ALL) {
                    if Self::scenario_card(column, results, kind, selected) {
                        clicked = Some(kind);
                    }
                }
            });
        }

        if let Some(kind) = clicked {
            self.selected = kind;
        }
    }

    fn scenario_tabs(&mut self, ui: &mut egui::Ui, results: &Results) {
        ui.horizontal_wrapped(|ui| {
            for kind in ScenarioKind::ALL {
                let label = format!(
                    "{}   {}",
                    kind.label(),
                    format_money(&results.scenario(kind).total_cost())
                );
                ui.selectable_value(&mut self.selected, kind, label)
                    .on_hover_text(kind.blurb());
            }
        });
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.selected.blurb()).small().weak());
    }

    fn scenario_card(
        ui: &mut egui::Ui,
        results: &Results,
        kind: ScenarioKind,
        selected: ScenarioKind,
    ) -> bool {
        let scenario = results.scenario(kind);
        let is_selected = kind == selected;

        let border = if is_selected {
            egui::Stroke::new(2.0f32, ui.visuals().strong_text_color())
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke
        };

        let card = egui::Frame::group(ui.style())
            .stroke(border)
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(CARD_MIN_HEIGHT);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    clicked = ui
                        .button(egui::RichText::new(kind.label()).strong())
                        .clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let note = if is_selected {
                            "Selected".to_owned()
                        } else {
                            format_delta(
                                &scenario.total_cost(),
                                &results.scenario(selected).total_cost(),
                            )
                        };
                        ui.label(egui::RichText::new(note).weak());
                    });
                });

                ui.label(
                    egui::RichText::new(format_money(&scenario.total_cost()))
                        .size(30.0)
                        .strong(),
                );
                ui.add_space(4.0);

                let travel = match &scenario.distance_km {
                    Some(d) => format!("Travel · {}", format_distance(d)),
                    None => "Travel".to_owned(),
                };
                card_row(ui, "Groceries", format_money(&scenario.grocery_cost()));
                card_row(
                    ui,
                    &travel,
                    scenario
                        .travel_cost
                        .as_ref()
                        .map_or_else(|| UNAVAILABLE.to_owned(), format_money),
                );
                card_row(
                    ui,
                    "Time",
                    scenario
                        .duration_min
                        .map_or_else(|| UNAVAILABLE.to_owned(), format_duration),
                );

                ui.add_space(6.0);
                let chip_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                ui.horizontal_wrapped(|ui| {
                    for stop in &scenario.stops {
                        egui::Frame::new()
                            .stroke(chip_stroke)
                            .corner_radius(8)
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(stop_name(stop));
                            });
                    }
                });

                clicked
            });

        let clicked_card = card
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(kind.blurb())
            .clicked();

        card.inner || clicked_card
    }

    fn plan_detail(ui: &mut egui::Ui, results: &Results, kind: ScenarioKind) {
        let scenario = results.scenario(kind);
        ui.heading(format!("{} plan, stop by stop", kind.label()));
        ui.add_space(8.0);

        if scenario.stops.is_empty() {
            ui.label(
                egui::RichText::new("No combination of stores in range can supply this list.")
                    .color(ui.visuals().warn_fg_color),
            );
            return;
        }

        let per_row = if ui.available_width() < STACK_BELOW_WIDTH {
            1
        } else {
            scenario.stops.len().min(2)
        };
        for (row, stops) in scenario.stops.chunks(per_row).enumerate() {
            ui.columns(per_row, |columns| {
                for (offset, (column, stop)) in columns.iter_mut().zip(stops).enumerate() {
                    let number = row * per_row + offset + 1;
                    column.push_id(number, |ui| Self::store_card(ui, number, stop));
                }
            });
            ui.add_space(12.0);
        }

        if let Some(note) = results.trade_off(kind) {
            egui::Frame::group(ui.style())
                .corner_radius(6)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(note);
                });
        }
    }

    fn store_card(ui: &mut egui::Ui, stop_number: usize, stop: &StoreStop) {
        let badge_fill = ui.visuals().strong_text_color();
        let badge_text = ui.visuals().panel_fill;

        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(badge_fill)
                .corner_radius(255)
                .inner_margin(egui::Margin::symmetric(7, 2))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(stop_number.to_string())
                            .small()
                            .strong()
                            .color(badge_text),
                    );
                });
            ui.label(egui::RichText::new(stop_name(stop)).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format_money(&stop.subtotal())).strong());
            });
        });
        if let Some(address) = &stop.address {
            ui.label(egui::RichText::new(address.as_str()).small().weak());
        }
        ui.add_space(4.0);

        egui::Frame::group(ui.style())
            .corner_radius(6)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for (index, item) in stop.items.iter().enumerate() {
                    if index > 0 {
                        ui.separator();
                    }
                    let mut name = if item.multiplier > 1 {
                        format!("{} × {}", item.name, item.multiplier)
                    } else {
                        item.name.clone()
                    };
                    if item.on_special {
                        name.push_str(" · special");
                    }
                    if item.needs_loyalty_card {
                        name.push_str(" · club price");
                    }
                    ui.horizontal(|ui| {
                        ui.label(name);
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| ui.label(format_money(&item.line_total())),
                        );
                    });
                }
            });
    }

    fn unresolved_banner(ui: &mut egui::Ui, items: &[UnresolvedItem]) {
        let warn = ui.visuals().warn_fg_color;

        egui::Frame::group(ui.style())
            .stroke(egui::Stroke::new(1.0f32, warn))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let heading = if items.len() == 1 {
                    "1 item could not be priced".to_owned()
                } else {
                    format!("{} items could not be priced", items.len())
                };
                ui.label(egui::RichText::new(heading).strong().color(warn));
                ui.add_space(2.0);
                for item in items {
                    ui.label(format!("{}  ({})", item.raw_text, item.reason.message()));
                }
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("These are not included in any of the totals below.")
                        .small()
                        .weak(),
                );
            });
    }

    fn loading(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.spinner();
            ui.add_space(8.0);
            ui.label("Comparing stores and routes...");
        });
    }

    fn message(ui: &mut egui::Ui, heading: &str, body: &str, colour: egui::Color32) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.label(egui::RichText::new(heading).heading().color(colour));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(body).weak());
        });
    }
}

fn card_row(ui: &mut egui::Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).strong());
        });
    });
}

fn stop_name(stop: &StoreStop) -> &str {
    stop.store_name
        .as_deref()
        .unwrap_or(brand_label(stop.chain))
}
