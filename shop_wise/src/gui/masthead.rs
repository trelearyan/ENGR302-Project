use std::sync::OnceLock;
use eframe::egui::{self, Color32, Stroke, Ui};
use super::AppData;

const INK: Color32 = Color32::from_rgb(0x0F, 0x3B, 0x45);
const FILL: Color32 = Color32::from_rgb(0xF5, 0xF1, 0xE8);
const LINE: Color32 = Color32::BLACK;
const MASTHEAD_HEIGHT: f32 = 60.0;
const LOGO_SIZE: f32 = 44.0;
const LOGO_GAP: f32 = 2.0;
const TEXT_SIZE: f32 = 24.0;
const GITLAB_ICON_SIZE: f32 = 50.0;
const MENU_ICON_SIZE: f32 = 22.0;
const LEFT_EDGE_MARGIN: f32 = -25.0;
const RIGHT_EDGE_MARGIN: f32 = 16.0;
const ICON_REST_ALPHA: u8 = 200;
const ICON_HOVER_ALPHA: u8 = 255;
const ICON_PRESSED_ALPHA: u8 = 150;
const LOGO_NUDGE_Y: f32 = 0.0;
const REPO_URL: &str = "https://gitlab.ecs.vuw.ac.nz/course-work/engr301/2026/project1/team5/shopwise";
const FULL_UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

macro_rules! cached_texture {
    ($ctx:expr, $name:literal, $bytes:expr) => {{
        static CACHED: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

        CACHED
            .get_or_init(|| decode_texture($ctx, $name, $bytes))
            .clone()
    }};
}

fn interact_alpha(response: &egui::Response) -> u8 {
    if response.is_pointer_button_down_on() {
        ICON_PRESSED_ALPHA
    } else if response.hovered() {
        ICON_HOVER_ALPHA
    } else {
        ICON_REST_ALPHA
    }
}

impl AppData {
    pub(crate) fn masthead(&mut self, ui: &mut Ui) {
        let logo = cached_texture!(
            ui.ctx(),
            "shopwise-logo",
            include_bytes!("../../assets/logo.png")
        );
        let git_icon = cached_texture!(
            ui.ctx(),
            "shopwise-repo",
            include_bytes!("../../assets/gitlab-logo.png")
        );
        let menu_collapse_icon = cached_texture!(
            ui.ctx(),
            "filters-colllapse ",
            include_bytes!("../../assets/arrow-left.png")
        );
        let menu_expand_icon = cached_texture!(
            ui.ctx(),
            "filters-expand",
            include_bytes!("../../assets/arrow-right.png")
        );

        egui::Panel::top("masthead")
            .exact_size(MASTHEAD_HEIGHT)
            .frame(
                egui::Frame::default()
                    .fill(FILL)
                    .inner_margin(egui::Margin::symmetric(24, 0)),
            )
            .show_inside(ui, |ui| {
                let rect = ui.max_rect();
                let centre_y = rect.center().y;

                let button_size = egui::vec2(44.0, 40.0);

                // Left menu button
                let menu_rect = egui::Rect::from_center_size(
                    egui::pos2(rect.left() + LEFT_EDGE_MARGIN + button_size.x / 2.0, centre_y),
                    button_size,
                );

                let menu_hover_text = if self.filters_collapsed {
                    "Show filters panel"
                    }else {
                    "Hide filters panel"
                };

                let menu = ui
                    .interact(
                        menu_rect,
                        ui.id().with("filters-menu"),
                        egui::Sense::click(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(menu_hover_text);

                let menu_alpha = interact_alpha(&menu);
                let menu_icon = if self.filters_collapsed {
                    &menu_expand_icon
                } else {
                    &menu_collapse_icon
                };

                match menu_icon {
                    Some(icon) => {
                        let icon_rect = egui::Rect::from_center_size(
                            menu_rect.center(),
                            egui::Vec2::splat(MENU_ICON_SIZE),
                        );

                        ui.painter().image(
                            icon.id(),
                            icon_rect,
                            FULL_UV,
                            Color32::from_white_alpha(menu_alpha),
                        );
                    },
                    &None => todo!()
                    // Falls back to a text glyph if either PNG failed to
                    // decode, so a bad asset degrades instead of leaving a
                    // dead, invisible button.

                }

                if menu.clicked() {
                    // TODO: collapse left panel
                }

                // Right button: opens the repository.
                let gitlab_rect = egui::Rect::from_center_size(
                    egui::pos2(
                        rect.right() - RIGHT_EDGE_MARGIN - button_size.x / 2.0,
                        centre_y,
                    ),
                    button_size,
                );

                let git_repo = ui
                    .interact(
                        gitlab_rect,
                        ui.id().with("git-account"),
                        egui::Sense::click(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Open the GitLab repository");

                if let Some(icon) = &git_icon {
                    let alpha = if git_repo.is_pointer_button_down_on() {
                        ICON_PRESSED_ALPHA
                    } else if git_repo.hovered() {
                        u8::MAX
                    } else {
                        ICON_REST_ALPHA
                    };

                    let icon_rect = egui::Rect::from_center_size(
                        gitlab_rect.center(),
                        egui::Vec2::splat(GITLAB_ICON_SIZE),
                    );

                    ui.painter().image(
                        icon.id(),
                        icon_rect,
                        FULL_UV,
                        Color32::from_white_alpha(alpha),
                    );
                }

                if git_repo.clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(REPO_URL));
                }

                let galley = ui.painter().layout_no_wrap(
                    "ShopWise".to_owned(),
                    egui::FontId::proportional(TEXT_SIZE),
                    INK,
                );

                let logo_advance = if logo.is_some() {
                    LOGO_SIZE + LOGO_GAP
                } else {
                    0.0
                };

                let group_width = logo_advance + galley.size().x;
                let group_left = rect.center().x - group_width / 2.0;
                let group_centre_y = centre_y + LOGO_NUDGE_Y;

                if let Some(logo) = &logo {
                    let logo_rect = egui::Rect::from_min_size(
                        egui::pos2(group_left, group_centre_y - LOGO_SIZE / 2.0),
                        egui::Vec2::splat(LOGO_SIZE),
                    );

                    ui.painter()
                        .image(logo.id(), logo_rect, FULL_UV, Color32::WHITE);
                }

                let text_pos = egui::pos2(
                    group_left + logo_advance,
                    group_centre_y - galley.size().y / 2.0,
                );

                ui.painter().galley(text_pos, galley, INK);

                ui.painter()
                    .hline(rect.x_range(), rect.bottom(), Stroke::new(1.0_f32, LINE));
            });
    }
}

fn decode_texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<egui::TextureHandle> {
    let decoded = match image::load_from_memory(bytes) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            log::error!("could not decode {name}: {error}");
            return None;
        }
    };

    let size = [decoded.width() as usize, decoded.height() as usize];
    let colour_image = egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());

    Some(ctx.load_texture(name, colour_image, egui::TextureOptions::LINEAR))
}