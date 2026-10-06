use eframe::egui;

use crate::{
    gui::ShowableWidget,
    price_calculator::results::{
        Results, ResultsState, Scenario, ScenarioKind, StoreStop, UnresolvedItem, brand_label,
        format_distance, format_duration, format_money, format_or_unavailable,
    },
};

const STACK_BELOW_WIDTH: f32 = 760.0;
const CARD_MIN_HEIGHT: f32 = 170.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PanelLayout {
    #[default]
    Cards,
    Tabs,
}

#[derive(Clone, Debug, Default)]
pub struct OutputRoutesData {
    pub selected: ScenarioKind,
    pub layout: PanelLayout,
    pub results: ResultsState,
}

impl ShowableWidget for OutputRoutesData {
    fn show(&mut self, ui: &mut egui::Ui) {
        let weak_colour = ui.visuals().weak_text_color();
        let error_colour = ui.visuals().error_fg_color;

        match &self.results {
            ResultsState::Idle => message(
                ui,
                "Nothing to show yet",
                "Add at least one item to your shopping list, set your location, then run a search.",
                weak_colour,
            ),
            ResultsState::Loading => loading(ui),
            ResultsState::Failed(reason) => message(
                ui,
                "That search could not be completed",
                reason,
                error_colour,
            ),
            ResultsState::Ready(results) => ready(ui, results, self.layout, &mut self.selected),
        }
    }
}

fn ready(ui: &mut egui::Ui, results: &Results, layout: PanelLayout, selected: &mut ScenarioKind) {
    ui.horizontal_wrapped(|ui| {
        ui.label(caption("Starting from"));
        ui.label(
            egui::RichText::new(results.origin_label.as_str())
                .small()
                .strong(),
        );
    });
    ui.add_space(8.0);

    if !results.unresolved.is_empty() {
        unresolved_banner(ui, &results.unresolved);
        ui.add_space(8.0);
    }

    match layout {
        PanelLayout::Cards => scenario_cards(ui, results, selected),
        PanelLayout::Tabs => scenario_tabs(ui, results, selected),
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    plan_detail(ui, results.scenario(*selected));
}

fn scenario_cards(ui: &mut egui::Ui, results: &Results, selected: &mut ScenarioKind) {
    let current = *selected;
    let mut clicked: Option<ScenarioKind> = None;

    if ui.available_width() < STACK_BELOW_WIDTH {
        for kind in ScenarioKind::ALL {
            if scenario_card(ui, results, kind, kind == current) {
                clicked = Some(kind);
            }
            ui.add_space(8.0);
        }
    } else {
        ui.columns(ScenarioKind::ALL.len(), |columns| {
            for (column, kind) in columns.iter_mut().zip(ScenarioKind::ALL) {
                if scenario_card(column, results, kind, kind == current) {
                    clicked = Some(kind);
                }
            }
        });
    }

    if let Some(kind) = clicked {
        *selected = kind;
    }
}

fn scenario_tabs(ui: &mut egui::Ui, results: &Results, selected: &mut ScenarioKind) {
    ui.horizontal_wrapped(|ui| {
        for kind in ScenarioKind::ALL {
            let label = format!(
                "{}   {}",
                kind.label(),
                format_money(&results.scenario(kind).total_cost())
            );
            ui.selectable_value(selected, kind, label)
                .on_hover_text(kind.blurb());
        }
    });
    ui.add_space(4.0);
    ui.label(caption(selected.blurb()));
}

fn scenario_card(ui: &mut egui::Ui, results: &Results, kind: ScenarioKind, selected: bool) -> bool {
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
            ui.label(caption(kind.blurb()));

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format_money(&scenario.total_cost()))
                    .size(26.0)
                    .strong(),
            );
            ui.label(caption("groceries plus petrol"));

            ui.add_space(8.0);
            egui::Grid::new(kind.label())
                .num_columns(2)
                .spacing([12.0, 3.0])
                .show(ui, |ui| {
                    ui.label(caption("Groceries"));
                    ui.label(egui::RichText::new(format_money(&scenario.grocery_cost())).small());
                    ui.end_row();

                    ui.label(caption("Travel"));
                    ui.label(caption(format_or_unavailable(
                        scenario.travel_cost.as_ref(),
                        format_money,
                    )));
                    ui.end_row();

                    ui.label(caption("Distance"));
                    ui.label(caption(format_or_unavailable(
                        scenario.distance_km.as_ref().zip(scenario.duration_min),
                        |(distance, minutes)| {
                            format!(
                                "{} ({})",
                                format_distance(distance),
                                format_duration(minutes)
                            )
                        },
                    )));
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
        stat(ui, "Groceries", &format_money(&scenario.grocery_cost()));
        stat(ui, "Total", &format_money(&scenario.total_cost()));
        stat(ui, "Items", &scenario.item_count().to_string());
        stat(
            ui,
            "Petrol",
            &format_or_unavailable(scenario.travel_cost.as_ref(), format_money),
        );
        stat(
            ui,
            "Distance",
            &format_or_unavailable(scenario.distance_km.as_ref(), format_distance),
        );
        stat(
            ui,
            "Driving time",
            &format_or_unavailable(scenario.duration_min, format_duration),
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

    for (index, stop) in scenario.stops.iter().enumerate() {
        ui.push_id(index, |ui| {
            store_card(ui, index + 1, stop);
        });
        ui.add_space(8.0);
    }
}

fn store_card(ui: &mut egui::Ui, stop_number: usize, stop: &StoreStop) {
    let positive = positive_color(ui);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.horizontal_wrapped(|ui| {
            ui.label(caption(format!("Stop {stop_number}")));
            ui.label(egui::RichText::new(stop.display_name()).strong());
            ui.label(caption(brand_label(stop.chain)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format_money(&stop.subtotal())).strong());
            });
        });
        if let Some(address) = &stop.address {
            ui.label(caption(address.as_str()));
        }
        ui.add_space(6.0);

        egui::Grid::new("items")
            .num_columns(4)
            .striped(true)
            .spacing([14.0, 4.0])
            .show(ui, |ui| {
                for heading in ["Item", "Qty", "Unit", "Line total"] {
                    ui.label(caption(heading));
                }
                ui.end_row();

                for item in &stop.items {
                    ui.horizontal(|ui| {
                        ui.label(item.name.as_str());
                        if item.on_special {
                            ui.label(egui::RichText::new("Special").small().color(positive));
                        }
                        if item.needs_loyalty_card {
                            ui.label(caption("Club price"));
                        }
                    });
                    ui.label(item.quantity.as_str());
                    ui.label(format_money(&item.unit_price));
                    ui.label(format_money(&item.line_total()));
                    ui.end_row();
                }
            });
    });
}

fn stat(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(caption(label));
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
            ui.label(caption(
                "These are not included in any of the totals below.",
            ));
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

/// Small, faded text used for labels and secondary details.
fn caption(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).small().weak()
}

fn positive_color(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(0x7d, 0xd3, 0x87)
    } else {
        egui::Color32::from_rgb(0x1b, 0x5e, 0x20)
    }
}
