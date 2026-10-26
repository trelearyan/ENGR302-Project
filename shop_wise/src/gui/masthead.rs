use std::sync::OnceLock;
use eframe::egui::{self, Color32, Stroke, Ui};
use super::AppData;

const FILL: Color32 = Color32::from_rgb(0xF5, 0xF1, 0xE8);
const LINE: Color32 = Color32::from_rgb(0, 0, 0);
const MASTHEAD_HEIGHT: f32 = 60.0;
const LOGO_HEIGHT: f32 = 36.0;
const GITLAB_ICON_SIZE: f32 = 50.0;
const MENU_ICON_SIZE: f32 = 22.0;
const LEFT_EDGE_MARGIN: f32 = -25.0;
const RIGHT_EDGE_MARGIN: f32 = 16.0;
const ICON_REST_ALPHA: u8 = 200;
const ICON_HOVER_ALPHA: u8 = 255;
const ICON_PRESSED_ALPHA: u8 = 150;
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
        let menu_collapse_icon_light = cached_texture!(
            ui.ctx(),
            "filters-collapse-light",
            include_bytes!("../../assets/arrow-left-light.png")
        );
        let menu_expand_icon_light = cached_texture!(
            ui.ctx(),
            "filters-expand-light",
            include_bytes!("../../assets/arrow-right-light.png")
        );
        let logo_dark_mode = cached_texture!(
            ui.ctx(),
            "shopwise-logo-dark-mode",
            include_bytes!("../../assets/logo-dark-mode.png")
        );

        let panel = egui::Panel::top("masthead")
            .exact_size(MASTHEAD_HEIGHT)
            .frame(
                egui::Frame::default()
                    .fill(if ui.visuals().dark_mode {
                        ui.visuals().panel_fill
                    } else {
                        FILL
                    })
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
                let dark_mode = ui.visuals().dark_mode;
                let menu_icon = match (self.filters_collapsed, dark_mode) {
                    (true, false) => &menu_expand_icon,
                    (false, false) => &menu_collapse_icon,
                    (true, true) => &menu_expand_icon_light,
                    (false, true) => &menu_collapse_icon_light,
                };

                if let Some(icon) = menu_icon {
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
                }

                if menu.clicked() {
                    self.filters_collapsed = !self.filters_collapsed;
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

                let logo = if ui.visuals().dark_mode {&logo_dark_mode} else { &logo };
                if let Some(logo) = &logo {
                    let [width, height] = logo.size();
                    let logo_width = LOGO_HEIGHT * width as f32 / height as f32;

                    let logo_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.center().x, centre_y),
                        egui::vec2(logo_width, LOGO_HEIGHT),
                    );

                    ui.painter()
                        .image(logo.id(), logo_rect, FULL_UV, Color32::WHITE);
                }
            });
        let line_width = 1.0_f32;
        let bar = panel.response.rect;
        ui.painter()
            .hline(bar.x_range(), bar.bottom()-line_width/2.0, Stroke::new(line_width, LINE));
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