use eframe::egui;
use egui_extras::{Column, TableBuilder};
use routx::{Graph, KDTree};
use std::ops::Deref;

use crate::{
    gui::ShowableWidget,
    price_calculator::results::{
        Results, ResultsState, Scenario, ScenarioKind, StoreStop, UNAVAILABLE, UnresolvedItem,
        brand_label, format_delta, format_distance, format_duration, format_money,
    },
    route_planner::{OPTIONS, make_graph},
};

impl ShowableWidget for OutputRoutesData {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.showold(ui);
    }
}

const STACK_BELOW_WIDTH: f32 = 760.0;
const CARD_MIN_HEIGHT: f32 = 140.0;
const MONEY_COLUMN_WIDTH: f32 = 72.0;
const BADGE_SIZE: f32 = 22.0;
const TICK_SIZE: f32 = 18.0;
const ORANGE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x7A, 0x3D);

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
    pub graph: Graph,
    pub tree: KDTree,
}

impl Default for OutputRoutesData {
    fn default() -> Self {
        log::debug!("route graph initialization begin");

        let graph = make_graph(&OPTIONS);
        let tree = KDTree::build_from_graph(&graph).unwrap();

        log::debug!("route graph initialized");

        log::debug!("route graph has {:?} nodes", graph.len());

        Self {
            selected: ScenarioKind::Best,
            layout: PanelLayout::Cards,
            results: ResultsState::Idle,
            graph,
            tree,
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
            ui.label(egui::RichText::new("Starting from:").small().strong());
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
            egui::Stroke::new(3.0f32, egui::Color32::from_rgb(0xFF, 0x7A, 0x3D))
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke
        };

        let card = egui::Frame::group(ui.style())
            .stroke(border)
            .corner_radius(8)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(CARD_MIN_HEIGHT);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    clicked = ui
                        .button(
                            egui::RichText::new(format!("{} {}", kind.icon(), kind.label()))
                                .strong(),
                        )
                        .clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_selected {
                            selected_tick(ui);
                        } else if let Some(delta) = format_delta(
                            &scenario.total_cost(),
                            &results.scenario(selected).total_cost(),
                        ) {
                            ui.label(egui::RichText::new(delta).weak());
                        }
                    });
                });

                ui.label(
                    egui::RichText::new(format_money(&scenario.total_cost()))
                        .size(30.0)
                        .strong(),
                );
                ui.add_space(4.0);

                card_row(ui, "Groceries", format_money(&scenario.grocery_cost()));
                card_row(ui, "Travel", travel_text(scenario));
                card_row(ui, "Distance", distance_text(scenario));
                card_row(ui, "Time", time_text(scenario));

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
        let _heading = ui.label(
            egui::RichText::new(format!("{} plan", kind.label()))
                .size(20.0)
                .strong(),
        );
        ui.add_space(12.0);

        let stats = [
            ("Groceries", format_money(&scenario.grocery_cost())),
            ("Travel", travel_text(scenario)),
            ("Total", format_money(&scenario.total_cost())),
            ("Items", scenario.item_count().to_string()),
            ("Distance", distance_text(scenario)),
            ("Driving time", time_text(scenario)),
        ];
        egui::Frame::group(ui.style())
            .corner_radius(8)
            .inner_margin(4)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let per_row = if ui.available_width() < STACK_BELOW_WIDTH {
                    3
                } else {
                    stats.len()
                };
                for row in stats.chunks(per_row) {
                    ui.columns(per_row, |columns| {
                        for (column, (label, value)) in columns.iter_mut().zip(row) {
                            column.vertical_centered(|ui| {
                                ui.label(*label);
                                ui.label(egui::RichText::new(value).size(16.0).strong());
                            });
                        }
                    });
                }
            });
        ui.add_space(12.0);

        if scenario.stops.is_empty() {
            ui.label(
                egui::RichText::new("No combination of stores in range can supply this list.")
                    .color(ui.visuals().warn_fg_color),
            );
            return;
        }

        let column_count = if ui.available_width() < STACK_BELOW_WIDTH {
            1
        } else {
            scenario.stops.len().min(2)
        };
        let item_counts: Vec<usize> = scenario.stops.iter().map(|stop| stop.items.len()).collect();
        let stacks = stack_stops(&item_counts, column_count);
        ui.columns(column_count, |columns| {
            for (column, stack) in columns.iter_mut().zip(&stacks) {
                for &index in stack {
                    let number = index + 1;
                    column.push_id(number, |ui| Self::store_card(ui, number, &scenario.stops[index]));
                    column.add_space(12.0);
                }
            }
        });

        if let Some(note) = results.trade_off(kind) {
            egui::Frame::group(ui.style())
                .corner_radius(6)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| ui.label(note));
                });
        }
    }

    fn store_card(ui: &mut egui::Ui, stop_number: usize, stop: &StoreStop) {
        let badge_fill = ui.visuals().strong_text_color();
        let badge_text = ui.visuals().panel_fill;

        ui.horizontal(|ui| {
            let (badge, _) =
                ui.allocate_exact_size(egui::vec2(BADGE_SIZE, BADGE_SIZE), egui::Sense::hover());
            ui.painter()
                .circle_filled(badge.center(), BADGE_SIZE / 2.0, badge_fill);
            ui.painter().text(
                badge.center(),
                egui::Align2::CENTER_CENTER,
                stop_number.to_string(),
                egui::FontId::monospace(14.0),
                badge_text,
            );
            ui.label(egui::RichText::new(stop_name(stop)).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format_money(&stop.subtotal())).strong());
            });
        });
        if let Some(address) = &stop.address {
            ui.label(egui::RichText::new(address.as_str()).small().weak());
        }
        ui.add_space(4.0);

        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 8.0;
        let right = egui::Layout::right_to_left(egui::Align::Center);
        egui::Frame::group(ui.style())
            .corner_radius(6)
            .show(ui, |ui| {
                // An exact width, worked out every frame: a `remainder` column never shrinks
                // below its widest label, so long names pushed the table into the next card.
                let item_width = (ui.available_width()
                    - 3.0 * (MONEY_COLUMN_WIDTH + ui.spacing().item_spacing.x))
                    .max(0.0);
                TableBuilder::new(ui)
                    .striped(true)
                    .vscroll(false)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::exact(item_width))
                    .columns(Column::exact(MONEY_COLUMN_WIDTH), 3)
                    .header(row_height, |mut header| {
                        header.col(|ui| {
                            ui.strong("Item");
                        });
                        for title in ["Qty", "Unit", "Line total"] {
                            header.col(|ui| {
                                ui.with_layout(right, |ui| ui.strong(title));
                            });
                        }
                    })
                    .body(|mut body| {
                        for item in &stop.items {
                            let mut name = item.name.clone();
                            if item.on_special {
                                name.push_str(" · special");
                            }
                            if item.needs_loyalty_card {
                                name.push_str(" · club price");
                            }
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.add(egui::Label::new(name).truncate());
                                });
                                for value in [
                                    item.quantity.clone(),
                                    format_money(&item.unit_price),
                                    format_money(&item.line_total()),
                                ] {
                                    row.col(|ui| {
                                        ui.with_layout(right, |ui| ui.label(value));
                                    });
                                }
                            });
                        }
                    });
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

fn travel_text(scenario: &Scenario) -> String {
    scenario
        .travel_cost
        .as_ref()
        .map_or_else(|| UNAVAILABLE.to_owned(), format_money)
}

fn distance_text(scenario: &Scenario) -> String {
    scenario
        .distance_km
        .as_ref()
        .map_or_else(|| UNAVAILABLE.to_owned(), format_distance)
}

fn time_text(scenario: &Scenario) -> String {
    scenario
        .duration_min
        .map_or_else(|| UNAVAILABLE.to_owned(), format_duration)
}
fn selected_tick(ui: &mut egui::Ui) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(TICK_SIZE, TICK_SIZE), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, ORANGE);
    let point = |x: f32, y: f32| rect.left_top() + egui::vec2(x, y) * TICK_SIZE;
    painter.add(egui::Shape::line(
        vec![point(0.25, 0.52), point(0.43, 0.70), point(0.76, 0.32)],
        egui::Stroke::new(2.0f32, egui::Color32::WHITE),
    ));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, "Selected"));
    response.on_hover_text("Selected");
}
fn card_row(ui: &mut egui::Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(label);
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

/// Splits stop indices into columns, putting each stop under whichever column has the
/// fewest rows so far. A long first stop then sits beside the shorter stops stacked together.
fn stack_stops(item_counts: &[usize], column_count: usize) -> Vec<Vec<usize>> {
    // Each card also has a store name row, an address row and a table header.
    const CARD_EXTRA_ROWS: usize = 3;

    let mut stacks = vec![Vec::new(); column_count];
    let mut rows = vec![0; column_count];
    for (index, count) in item_counts.iter().enumerate() {
        let shortest = (0..column_count).min_by_key(|&c| rows[c]).unwrap_or(0);
        rows[shortest] += count + CARD_EXTRA_ROWS;
        stacks[shortest].push(index);
    }
    stacks
}

#[cfg(test)]
mod tests {
    use super::stack_stops;

    #[test]
    fn long_first_stop_stacks_others_beside_it() {
        assert_eq!(stack_stops(&[11, 6, 7], 2), vec![vec![0], vec![1, 2]]);
    }

    #[test]
    fn short_first_stop_takes_the_third() {
        assert_eq!(stack_stops(&[3, 10, 4], 2), vec![vec![0, 2], vec![1]]);
    }

    #[test]
    fn one_column_keeps_order() {
        assert_eq!(stack_stops(&[5, 2, 8], 1), vec![vec![0, 1, 2]]);
    }
}
