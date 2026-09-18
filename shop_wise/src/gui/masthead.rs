use eframe::egui::{
    self, pos2, vec2, Color32, Sense, Shape, Stroke, Ui, Vec2,
};

use crate::AppData;

const INK: Color32 = Color32::from_rgb(0x0F, 0x3B, 0x45);
const FILL: Color32 = Color32::from_rgb(0xF5, 0xF1, 0xE8);
const LINE: Color32 = Color32::from_rgb(0xDE, 0xD8, 0xCB);

impl AppData {
    pub(crate) fn masthead(&mut self, ui: &mut Ui) {
        let logo_texture = self
            .logo_texture
            .get_or_insert_with(|| load_logo(ui.ctx()))
            .clone();

        egui::Panel::top("masthead")
            .exact_size(60.0)
            .frame(
                egui::Frame::default()
                    .fill(FILL)
                    .inner_margin(egui::Margin::symmetric(24, 0)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    // Grocery basket logo
                    ui.add(
                        egui::Image::new(&logo_texture)
                            .fit_to_exact_size(egui::vec2(60.0, 60.0)),
                    );

                    ui.label(
                        egui::RichText::new("ShopWise")
                            .size(30.0)
                            .color(INK)
                            .strong(),
                    );
                });

                // Subtle divider along the bottom
                let rect = ui.max_rect();
                ui.painter().hline(
                    rect.x_range(),
                    rect.bottom(),
                    Stroke::new(1.0, LINE),
                );
            });
    }
}

fn load_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let bytes = include_bytes!("../../assets/logo.png");

    let image = image::load_from_memory(bytes)
        .expect("Failed to load ShopWise logo")
        .to_rgba8();

    let size = [
        image.width() as usize,
        image.height() as usize,
    ];

    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        size,
        image.as_raw(),
    );

    ctx.load_texture(
        "logo",
        color_image,
        egui::TextureOptions::LINEAR,
    )
}