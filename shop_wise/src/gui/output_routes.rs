use std::ops::Deref;
use eframe::egui;

use crate::{
    gui::ShowableWidget,
    price_calculator::results::{
        Results, ResultsState, Scenario, ScenarioKind, StoreStop, UNAVAILABLE, UnresolvedItem,
        brand_label, format_distance, format_duration, format_money,
    },
};

impl ShowableWidget for OutputRoutesData {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.showold(ui);
    }
}

const STACK_BELOW_WIDTH: f32 = 760.0;
const CARD_MIN_HEIGHT: f32 = 170.0;

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

        Self::plan_detail(ui, results.scenario(self.selected));
    }

    fn scenario_cards(&mut self, ui: &mut egui::Ui, results: &Results) {
        let selected = self.selected;
        let mut clicked: Option<ScenarioKind> = None;

        if ui.available_width() < STACK_BELOW_WIDTH {
            for kind in ScenarioKind::ALL {
                if Self::scenario_card(ui, results, kind, kind == selected) {
                    clicked = Some(kind);
                }
                ui.add_space(8.0);
            }
        } else {
            ui.columns(ScenarioKind::ALL.len(), |columns| {
                for (column, kind) in columns.iter_mut().zip(ScenarioKind::ALL) {
                    if Self::scenario_card(column, results, kind, kind == selected) {
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
        selected: bool,
    ) -> bool {
        let scenario = results.scenario(kind);

        let accent = ui.visuals().selection.stroke.color;
        let faint = ui.visuals().faint_bg_color;
        let positive = positive_color(ui);

        let mut frame = egui::Frame::group(ui.style());
        if selected {
            frame = frame.fill(faint).stroke(egui::Stroke::new(2.0f32, accent));
        }

        let card = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(CARD_MIN_HEIGHT);
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(kind.label()).heading());
                    if kind == ScenarioKind::Cheapest
                        && let Some(saving) = results.headline_saving()
                    {
                        ui.label(
                            egui::RichText::new(format!("saves {}", format_money(&saving)))
                                .small()
                                .strong()
                                .color(positive),
                        );
                    }
                });
                ui.label(egui::RichText::new(kind.blurb()).small().weak());

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format_money(&scenario.total_cost()))
                        .size(26.0)
                        .strong(),
                );
                ui.label(egui::RichText::new("groceries plus petrol").small().weak());

                ui.add_space(8.0);
                egui::Grid::new(kind.label())
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Groceries").small().weak());
                        ui.label(
                            egui::RichText::new(format_money(&scenario.grocery_cost())).small(),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Travel").small().weak());
                        ui.label(
                            egui::RichText::new(match &scenario.travel_cost {
                                Some(c) => format_money(c),
                                None => UNAVAILABLE.to_owned(),
                            })
                            .small()
                            .weak(),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Distance").small().weak());
                        ui.label(
                            egui::RichText::new(
                                match (&scenario.distance_km, scenario.duration_min) {
                                    (Some(d), Some(m)) => {
                                        format!("{} ({})", format_distance(d), format_duration(m))
                                    }
                                    _ => UNAVAILABLE.to_owned(),
                                },
                            )
                            .small()
                            .weak(),
                        );
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label(egui::RichText::new(scenario.store_summary()).strong());

                ui.add_space(6.0);
                let label = if selected {
                    "Showing this plan"
                } else {
                    "Show this plan"
                };
                ui.selectable_label(selected, label).clicked()
            })
            .inner
        });

        let clicked_card = card
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked();

        card.inner || clicked_card
    }

    fn plan_detail(ui: &mut egui::Ui, scenario: &Scenario) {
        ui.heading(format!("{} plan", scenario.kind.label()));
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            Self::stat(ui, "Groceries", &format_money(&scenario.grocery_cost()));
            Self::stat(ui, "Total", &format_money(&scenario.total_cost()));
            Self::stat(ui, "Items", &scenario.item_count().to_string());
            Self::stat(
                ui,
                "Petrol",
                &match &scenario.travel_cost {
                    Some(c) => format_money(c),
                    None => UNAVAILABLE.to_owned(),
                },
            );
            Self::stat(
                ui,
                "Distance",
                &match &scenario.distance_km {
                    Some(d) => format_distance(d),
                    None => UNAVAILABLE.to_owned(),
                },
            );
            Self::stat(
                ui,
                "Driving time",
                &match scenario.duration_min {
                    Some(m) => format_duration(m),
                    None => UNAVAILABLE.to_owned(),
                },
            );
        });

        ui.add_space(10.0);

        if scenario.stops.is_empty() {
            ui.label(
                egui::RichText::new("No combination of stores in range can supply this list.")
                    .color(ui.visuals().warn_fg_color),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, stop) in scenario.stops.iter().enumerate() {
                    ui.push_id(index, |ui| {
                        Self::store_card(ui, index + 1, stop);
                    });
                    ui.add_space(8.0);
                }
            });
    }

    fn store_card(ui: &mut egui::Ui, stop_number: usize, stop: &StoreStop) {
        let positive = positive_color(ui);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("Stop {stop_number}"))
                        .small()
                        .weak(),
                );
                ui.label(
                    egui::RichText::new(
                        stop.store_name
                            .as_deref()
                            .unwrap_or(brand_label(stop.chain)),
                    )
                    .strong(),
                );
                ui.label(egui::RichText::new(brand_label(stop.chain)).small().weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format_money(&stop.subtotal())).strong());
                });
            });
            if let Some(address) = &stop.address {
                ui.label(egui::RichText::new(address.as_str()).small().weak());
            }
            ui.add_space(6.0);

            egui::Grid::new("items")
                .num_columns(4)
                .striped(true)
                .spacing([14.0, 4.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Item").small().weak());
                    ui.label(egui::RichText::new("Qty").small().weak());
                    ui.label(egui::RichText::new("Unit").small().weak());
                    ui.label(egui::RichText::new("Line total").small().weak());
                    ui.end_row();

                    for item in &stop.items {
                        ui.horizontal(|ui| {
                            ui.label(item.name.as_str());
                            if item.on_special {
                                ui.label(egui::RichText::new("Special").small().color(positive));
                            }
                            if item.needs_loyalty_card {
                                ui.label(egui::RichText::new("Club price").small().weak());
                            }
                        });
                        ui.label(item.quantity.to_string());
                        ui.label(format_money(&item.unit_price));
                        ui.label(format_money(&item.line_total()));
                        ui.end_row();
                    }
                });
        });
    }

    fn stat(ui: &mut egui::Ui, label: &str, value: &str) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).small().weak());
            ui.label(egui::RichText::new(value).size(17.0).strong());
        });
        ui.add_space(22.0);
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

fn positive_color(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(0x7d, 0xd3, 0x87)
    } else {
        egui::Color32::from_rgb(0x1b, 0x5e, 0x20)
    }
}
