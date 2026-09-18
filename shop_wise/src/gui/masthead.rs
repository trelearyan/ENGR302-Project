use std::sync::OnceLock;
use eframe::egui::{self, Color32, Stroke, Ui};
use super::AppData;

const INK: Color32 = Color32::from_rgb(0x0F, 0x3B, 0x45);
const FILL: Color32 = Color32::from_rgb(0xF5, 0xF1, 0xE8);
const LINE: Color32 = Color32::BLACK;

const BAR_HEIGHT: f32 = 60.0;
const LOGO_SIZE: f32 = 60.0;

impl AppData {
    pub(crate) fn masthead(&mut self, ui: &mut Ui) {
        let logo = logo_texture(ui.ctx());

        egui::TopBottomPanel::top("masthead")
            .exact_height(BAR_HEIGHT)
            .frame(
                egui::Frame::default()
                    .fill(FILL)
                    .inner_margin(egui::Margin::symmetric(24, 0)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some(logo) = &logo {
                        ui.add(
                            egui::Image::new(logo)
                                .fit_to_exact_size(egui::Vec2::splat(LOGO_SIZE)),
                        );
                    }

                    ui.label(
                        egui::RichText::new("ShopWise")
                            .size(24.0)
                            .color(INK)
                            .strong(),
                    );
                });

                let rect = ui.max_rect();
                ui.painter()
                    .hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, LINE));
            });
    }
}

fn logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    static CACHED: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

    CACHED
        .get_or_init(|| {
            let bytes = include_bytes!("../../assets/logo.png");

            let decoded = match image::load_from_memory(bytes) {
                Ok(image) => image.to_rgba8(),
                Err(error) => {
                    log::error!("could not decode the ShopWise logo: {error}");
                    return None;
                }
            };

            let size = [decoded.width() as usize, decoded.height() as usize];
            let colour_image =
                egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());

            Some(ctx.load_texture("shopwise-logo", colour_image, egui::TextureOptions::LINEAR))
        })
        .clone()
}