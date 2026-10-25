use eframe::egui::{self, Context, RichText, Ui};

const PRICES_AS_OF: &str = "09/10/2026?";

const REPO_URL: &str =
    "https://gitlab.ecs.vuw.ac.nz/course-work/engr301/2026/project1/team5/shopwise";
const CLUB_PLUS_URL: &str = "https://www.clubplus.co.nz";
const EVERYDAY_REWARDS_URL: &str = "https://www.woolworths.co.nz/help/everyday-rewards-faqs";
const OSM_COPYRIGHT_URL: &str = "https://www.openstreetmap.org/copyright";
const IRD_URL: &str = "https://www.ird.govt.nz/income-tax/income-tax-for-businesses-and-organisations/types-of-business-expenses/claiming-vehicle-expenses/kilometre-rates-2025-2026";


const POPUP_WIDTH: f32 = 450.0;
const POPUP_MAX_HEIGHT: f32 = 450.0;
const POPUP_SCREEN_MARGIN: f32 = 50.0;
const HEADING_SIZE: f32 = 16.0;
const TERM_WIDTH: f32 = 100.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FooterPage {
    #[default]
    None,
    About,
    Privacy,
    Terms,
    Data,
    Loyalty,
    Snake,
}

impl FooterPage {
    const LINKS: [Self; 6] = [
        Self::About,
        Self::Privacy,
        Self::Terms,
        Self::Data,
        Self::Loyalty,
        Self::Snake,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::None => "",
            Self::About => "About",
            Self::Privacy => "Privacy",
            Self::Terms => "Terms",
            Self::Data => "Data",
            Self::Loyalty => "Loyalty",
            Self::Snake => "Devs",
        }
    }

    fn body(self, ui: &mut Ui) {
        match self {
            Self::None => {}
            Self::About => about(ui),
            Self::Privacy => privacy(ui),
            Self::Terms => terms(ui),
            Self::Data => data(ui),
            Self::Loyalty => loyalty(ui),
            Self::Snake => snake(ui),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FooterData {
    open: FooterPage,
}

impl FooterData {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for (index, page) in FooterPage::LINKS.into_iter().enumerate() {
                if index > 0 {
                    ui.label(RichText::new("·").small().weak());
                }
                if ui.link(RichText::new(page.title()).small()).clicked() {
                    self.open = page;
                }
            }
        });

        ui.label(
            RichText::new(format!("ShopWise v{}", env!("CARGO_PKG_VERSION")))
                .small()
                .weak(),
        );
        self.window(ui.ctx());
    }

    fn window(&mut self, ctx: &Context) {
        let page = self.open;
        if page == FooterPage::None {
            return;
        }

        let mut open = true;
        let width = POPUP_WIDTH.min(ctx.content_rect().width() - POPUP_SCREEN_MARGIN);

        egui::Window::new(page.title())
            .id(egui::Id::new("footer_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(POPUP_MAX_HEIGHT)
                    .show(ui, |ui| {
                        ui.set_width(width);
                        page.body(ui);
                    });
            });
        if !open {
            self.open = FooterPage::None;
        }
    }
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(text).strong().size(HEADING_SIZE));
    ui.add_space(2.0);
}

fn paragraph(ui: &mut Ui, text: &str) {
    ui.label(text);
    ui.add_space(2.0);
}

fn term<R>(ui: &mut Ui, width: f32, term: &str, add_contents: impl FnOnce(&mut Ui) -> R) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.label(RichText::new(term).strong());
        });
        ui.vertical(add_contents);
    });
    ui.add_space(2.0);
}

fn about(ui: &mut Ui) {
    paragraph(
        ui,
        "ShopWise prices your whole shopping list across Pak'nSave, New World and \
         Woolworths near you, and counts the petrol it costs to get there.",
    );
    heading(ui, "Our three shopping scenarios:");
    term(ui, TERM_WIDTH, "Cheapest:", |ui| {
        ui.label("The lowest total, even if it means visiting more than one store.")
    });
    term(ui, TERM_WIDTH, "Fastest:", |ui| {
        ui.label("The shortest trip that still covers your entire shopping list.")
    });
    term(ui, TERM_WIDTH, "Best Value:", |ui| {
        ui.label(
            "A balance of the two. It can cost a little more than Cheapest when it \
             saves you a stop or a noticeable amount of driving.",
        )
    });
}

fn privacy(ui: &mut Ui) {
    paragraph(
        ui,
        "ShopWise runs entirely in your browser or on your computer. There is no \
         account, no server of ours, and no cookies, analytics or tracking.",
    );
    heading(ui, "What stays on your device:");
    paragraph(
        ui,
        "Your shopping list, your settings and your location are never sent to us. \
         Prices are a snapshot bundled with the app and read locally.",
    );
    heading(ui, "What leaves your device:");
    paragraph(
        ui,
        "When you search for your address, the text you type is sent to OpenStreetMap's \
         Nominatim service to find its coordinates. If you use your current location, \
         your browser asks your permission first, and the coordinates are looked up the \
         same way.",
    );
    heading(ui, "Files:");
    paragraph(
        ui,
        "Saved shopping lists go only where you choose to save them. Loaded files are \
         read on your device and are not uploaded.",
    );
}

fn terms(ui: &mut Ui) {
    paragraph(
        ui,
        "ShopWise is a university student project, built for ENGR 302 at \
         Victoria University of Wellington. It is provided as is, without any \
         warranty.",
    );
    heading(ui, "Prices are estimates");
    paragraph(
        ui,
        "Prices come from a snapshot and change often. Always check the price in store \
         before relying on it. Distances and petrol costs are estimates.",
    );
    heading(ui, "Not affiliated");
    paragraph(
        ui,
        "ShopWise is not affiliated with or endorsed by Foodstuffs, Woolworths New \
         Zealand or any supermarket. Store names and trademarks belong to their owners.",
    );
}

fn data(ui: &mut Ui) {
    paragraph(ui, "Where ShopWise's data comes from:");
    ui.add_space(4.0);
    term(ui, TERM_WIDTH, "Prices:", |ui| {
        ui.label("Collected from the supermarkets' public websites.");
        ui.label(RichText::new(format!("Prices as at {PRICES_AS_OF}")).weak());
    });
    term(ui, TERM_WIDTH, "Store locations:", |ui| {
        ui.label("Collected from each chain's public store finder.")
    });
    term(ui, TERM_WIDTH, "Petrol:", |ui| {
        ui.label("Per-kilometre rates follow the IRD vehicle rates for 2025 to 2026.");
        ui.hyperlink_to("IRD KM rates 25/26", IRD_URL);
    });
    term(ui, TERM_WIDTH, "Maps:", |ui| {
        ui.label("Address search uses Nominatim.");
        ui.hyperlink_to("Map data ©OpenStreetMap contributors", OSM_COPYRIGHT_URL);
    });
}

fn loyalty(ui: &mut Ui) {
    paragraph(
        ui,
        "Both loyalty programmes are free to join to unlock their members-only prices. \
         Tick the ones you have in the Supermarkets section.",
    );
    ui.add_space(4.0);
    programme(
        ui,
        "Club+",
        "Pak'nSave / New World",
        "One card for both stores, with Club Prices on selected products.",
        ("Join Club+", CLUB_PLUS_URL),
    );
    programme(
        ui,
        "Everyday Rewards",
        "Woolworths",
        "Woolworths' loyalty programme, with Member Prices on selected products.",
        ("Join Everyday Rewards", EVERYDAY_REWARDS_URL),
    );
}

fn programme(ui: &mut Ui, name: &str, stores: &str, text: &str, (link, url): (&str, &str)) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).strong().size(HEADING_SIZE));
                ui.label(RichText::new(stores).weak());
            });
            ui.label(text);
            ui.add_space(2.0);
            ui.hyperlink_to(link, url);
        });
    ui.add_space(6.0);
}

fn snake(ui: &mut Ui) {
    paragraph(ui, "This project was built by Team 5 as a part of ENGR301/302, 2026.");
    ui.add_space(4.0);
    for (team, names) in [
        ("Frontend:", &["Benjamin Khokgawe", "Ryan Treleaven"][..]),
        ("Backend:", &["Alexander Worth", "Samuel Smith", "Syon Krishna"][..]),
    ] {
        term(ui, TERM_WIDTH, team, |ui| {
            for name in names {
                ui.label(*name);
            }
        });
    }
    ui.add_space(6.0);
    ui.hyperlink_to("View our GitLab", REPO_URL);
}