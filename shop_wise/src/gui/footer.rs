use eframe::egui::{self, Context, RichText, Ui};

const PRICES_AS_AT: &str = "date?";

const REPO_URL: &str =
    "https://gitlab.ecs.vuw.ac.nz/course-work/engr301/2026/project1/team5/shopwise";
const CLUB_PLUS_URL: &str = "https://www.clubplus.co.nz";
const EVERYDAY_REWARDS_URL: &str = "https://www.woolworths.co.nz/help/everyday-rewards-faqs";

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

        egui::Window::new(page.title())
            .id(egui::Id::new("footer_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(420.0)
                    .show(ui, |ui| page.body(ui));
            });
        if !open {
            self.open = FooterPage::None;
        }
    }
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).strong());
}

fn para(ui: &mut Ui, text: &str) {
    ui.label(text);
    ui.add_space(4.0);
}

fn about(ui: &mut Ui) {
    para(
        ui,
        "ShopWise prices your whole shopping list across Pak'nSave, New World and \
         Woolworths near you, and counts the petrol it costs to get there.",
    );
    heading(ui, "The Three Shopping Scenarios");
    para(ui, "Cheapest: the lowest total, even if it means visiting more than one store.");
    para(ui, "Fastest: the shortest trip that still covers your entire shopping list.");
    para(
        ui,
        "Best Value: a balance of the two. It can cost a little more than Cheapest \
         when it saves you a stop or a noticeable amount of driving.",
    );
}

fn privacy(ui: &mut Ui) {
    para(
        ui,
        "ShopWise runs entirely in your browser or on your computer. There is no \
         account, no server of ours, and no cookies, analytics or tracking.",
    );
    heading(ui, "What stays on your device");
    para(
        ui,
        "Your shopping list, your settings and your location are never sent to us. \
         Prices are a snapshot bundled with the app and read locally.",
    );
    heading(ui, "What leaves your device");
    para(
        ui,
        "When you search for your address, the text you type is sent to OpenStreetMap's \
         Nominatim service to find its coordinates. If you use your current location, \
         your browser asks your permission first and the coordinates are looked up the \
         same way.",
    );
    heading(ui, "Files");
    para(
        ui,
        "Saved shopping lists go only where you choose to save them. Loaded files are \
         read on your device and are not uploaded.",
    );
}

fn terms(ui: &mut Ui) {
    para(
        ui,
        "ShopWise is a university student project, built for ENGR 302 at  \
         Victoria University of Wellington. It is provided as is, without any \
         warranty.",
    );
    heading(ui, "Prices are indicative");
    para(
        ui,
        "Prices come from a snapshot and change often. Always check the price in store \
         before relying on it. Distances and petrol costs are estimates.",
    );
    heading(ui, "Not affiliated");
    para(
        ui,
        "ShopWise is not affiliated with or endorsed by Foodstuffs, Woolworths New \
         Zealand or any supermarket. Store names and trademarks belong to their owners.",
    );
}

fn data(ui: &mut Ui) {
    heading(ui, "Prices");
    para(
        ui,
        &format!(
            "Collected from the supermarkets' public websites. Prices as at {PRICES_AS_AT}."
        ),
    );
    heading(ui, "Store locations");
    para(ui, "Collected from each chain's public store finder.");
    heading(ui, "Petrol");
    para(ui, "Per kilometre rates follow the IRD vehicle rates for 2025 to 2026.");
    heading(ui, "Maps and addresses");
    para(ui, "Address search uses Nominatim. Map data © OpenStreetMap contributors.");
}

fn loyalty(ui: &mut Ui) {
    para(
        ui,
        "Both loyalty programmes are free to join and unlock member-only prices. \
         You can TICK the ones you have in the Supermarkets section.",
    );

    heading(ui, "Club+");
    para(
        ui,
        "One card for Pak'nSave and New World, with Clbu Prices on selected products",
    );
    ui.hyperlink_to("Join Club+", CLUB_PLUS_URL);

    heading(ui, "Everyday Rewards");
    para(ui, "Woolworths' loyalty programme, with Member Prices on selected products.");
    ui.hyperlink_to("Join Everyday Rewards", EVERYDAY_REWARDS_URL);
}

fn snake(ui: &mut Ui) {
    para(ui, "This project was built by Team 5 as a part of ENGR301/302, 2026.");
    for (name, role) in [
        ("Alexander Worth:", "Route planner"),
        ("Samuel Smith:", "Price calculator, item resolver and Parser"),
        ("Syon Krishna:", "Database"),
        ("Benjamin Khokgawe:", "Input GUI"),
        ("Ryan Treleaven:", "Output GUI"),
    ] {
        ui.horizontal(|ui| {
            ui.label(RichText::new(name).strong());
            ui.label(RichText::new(role).weak());
        });
    }
    ui.add_space(8.0);
    ui.hyperlink_to("GitLab", REPO_URL);
}