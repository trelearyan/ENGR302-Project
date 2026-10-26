use std::rc::Rc;

use eframe::{
    CreationContext,
    egui::{self, Color32, Response, Ui},
    epaint::PathStroke,
};
use serde::Serialize;
use util::coordinate::Coordinate;
use walkers::{HttpTiles, Map, MapMemory, Plugin, Position, Projector, sources::OpenStreetMap};

pub struct MapData {
    current_pos: Option<(f32, f32)>,
    last_pos: Option<(f32, f32)>,
    current_displayed_route: Rc<[Coordinate]>,

    tiles: HttpTiles,
    map_memory: MapMemory,
}

impl MapData {
    pub fn new(creation_context: &CreationContext) -> Self {
        Self {
            tiles: HttpTiles::new(OpenStreetMap, creation_context.egui_ctx.clone()),
            map_memory: MapMemory::default(),
            current_pos: None,
            last_pos: None,
            current_displayed_route: Rc::new([]),
        }
    }
}

impl eframe::App for MapData {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut map = Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            walkers::lon_lat(174.77643827862266, -41.288175480492), // Wellington
        );
        if self.current_displayed_route.len() > 1 {
            println!("wahoo");

            map = map.with_plugin(RouteDrawingPlugin(self.current_displayed_route.clone()));
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct RouteDrawingPlugin(Rc<[Coordinate]>);

impl Plugin for RouteDrawingPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut Ui,
        response: &Response,
        projector: &Projector,
        map_memory: &MapMemory,
    ) {
        ui.painter().line(
            self.0
                .iter()
                .map(|coordinate| {
                    let coord = projector
                        .project(Position::from(coordinate.clone()))
                        .to_pos2();
                    coord
                })
                .collect(),
            PathStroke::new(3.0, Color32::RED),
        );
    }
}
