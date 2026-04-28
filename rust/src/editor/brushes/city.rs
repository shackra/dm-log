use crate::app::App;
use crate::editor::brush::Brush;
use crate::map::{Building, Cell, MapType};

/// Draws a named building rectangle on a City map.
/// Outer perimeter cells are locked after placement.
pub struct BuildingBrush {
    pub name: String,
    pub w: u16,
    pub h: u16,
}

impl BuildingBrush {
    pub fn new(name: impl Into<String>, w: u16, h: u16) -> Self {
        BuildingBrush { name: name.into(), w: w.max(3), h: h.max(3) }
    }

    pub fn grow(&mut self) {
        self.w += 1;
        self.h += 1;
    }

    pub fn shrink(&mut self) {
        self.w = (self.w - 1).max(3);
        self.h = (self.h - 1).max(3);
    }
}

impl Brush for BuildingBrush {
    fn name(&self) -> &str { "Building" }
    fn preview_char(&self) -> char { '#' }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;

        if app.current_map().map(|m| m.map_type != MapType::City).unwrap_or(true) {
            app.set_status("BuildingBrush only works on City maps.");
            return;
        }

        let building_id = uuid::Uuid::new_v4().to_string();
        let building = Building::new(
            building_id.clone(),
            self.name.clone(),
            col, row, self.w, self.h,
        );

        if let Some(map) = app.current_map_mut() {
            // Paint cells; lock perimeter
            if let Some(layer) = map.layer_mut(z) {
                for dy in 0..self.h {
                    for dx in 0..self.w {
                        let c = col + dx;
                        let r = row + dy;
                        let on_perim = dy == 0 || dy == self.h - 1
                            || dx == 0 || dx == self.w - 1;
                        if layer.cells.get(&(c, r)).map(|e| e.locked).unwrap_or(false) {
                            continue;
                        }
                        let mut cell = Cell::new('#').with_terrain("wall");
                        if on_perim {
                            cell = cell.locked();
                        } else {
                            cell = Cell::new('.').with_terrain("floor");
                        }
                        layer.cells.insert((c, r), cell);
                    }
                }
            }
            map.buildings.push(building);
        }

        app.set_status(format!("Building '{}' placed.", self.name));
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Paints road/path cells (`·`).
pub struct StreetBrush;

impl Brush for StreetBrush {
    fn name(&self) -> &str { "Street" }
    fn preview_char(&self) -> char { '\u{00B7}' } // ·

    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if !layer.cells.get(&(col, row)).map(|c| c.locked).unwrap_or(false) {
                    layer.cells.insert((col, row), Cell::new('\u{00B7}').with_terrain("road"));
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Fills an open plaza area (`.`).
pub struct PlazaBrush;

impl Brush for PlazaBrush {
    fn name(&self) -> &str { "Plaza" }
    fn preview_char(&self) -> char { '.' }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if !layer.cells.get(&(col, row)).map(|c| c.locked).unwrap_or(false) {
                    layer.cells.insert((col, row), Cell::new('.').with_terrain("plaza"));
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Draws box-drawing wall characters, auto-selecting based on direction.
pub struct WallBrush {
    /// Direction of travel used to pick the right box-drawing char.
    pub horizontal: bool,
}

impl WallBrush {
    pub fn new(horizontal: bool) -> Self {
        WallBrush { horizontal }
    }
}

impl Brush for WallBrush {
    fn name(&self) -> &str { "Wall" }
    fn preview_char(&self) -> char { if self.horizontal { '\u{2500}' } else { '\u{2502}' } }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        let ch = if self.horizontal { '\u{2500}' } else { '\u{2502}' }; // ─ │
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if !layer.cells.get(&(col, row)).map(|c| c.locked).unwrap_or(false) {
                    layer.cells.insert((col, row), Cell::new(ch).with_terrain("wall"));
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::map::{Layer, MapDef, MapType};
    use std::path::PathBuf;

    fn city_app() -> App {
        let mut map = MapDef::new("c1", "City", MapType::City, 60, 30);
        map.layers[0] = Layer::new(0, 0.0);
        App::new(PathBuf::from("/tmp"), Some(map))
    }

    #[test]
    fn building_brush_places_cells_and_records_building() {
        let mut app = city_app();
        app.cursor = (5, 3);
        let mut brush = BuildingBrush::new("Tavern", 6, 4);
        brush.on_confirm(&mut app);

        let map = app.current_map().unwrap();
        assert_eq!(map.buildings.len(), 1);
        assert_eq!(map.buildings[0].name, "Tavern");
        assert_eq!(map.buildings[0].x, 5);
        assert_eq!(map.buildings[0].y, 3);

        let layer = map.layer(0).unwrap();
        assert_eq!(layer.cells[&(5, 3)].ch, '#');
        assert!(layer.cells[&(5, 3)].locked);
        assert_eq!(layer.cells[&(6, 4)].ch, '.');
        assert!(!layer.cells[&(6, 4)].locked);
    }

    #[test]
    fn building_brush_rejected_on_non_city_map() {
        let mut map = MapDef::new("d1", "Dungeon", MapType::Dungeon, 40, 20);
        map.layers[0] = Layer::new(0, 0.0);
        let mut app = App::new(PathBuf::from("/tmp"), Some(map));
        app.cursor = (5, 5);
        let mut brush = BuildingBrush::new("Inn", 6, 4);
        brush.on_confirm(&mut app);
        assert!(app.current_map().unwrap().buildings.is_empty());
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn street_brush_places_road_cell() {
        let mut app = city_app();
        app.cursor = (10, 10);
        let mut brush = StreetBrush;
        brush.on_confirm(&mut app);
        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(10, 10)].ch, '\u{00B7}');
        assert_eq!(layer.cells[&(10, 10)].terrain, "road");
    }

    #[test]
    fn building_brush_does_not_overwrite_locked_cell() {
        let mut app = city_app();
        {
            let map = app.current_map_mut().unwrap();
            let layer = map.layer_mut(0).unwrap();
            layer.cells.insert((6, 4), crate::map::Cell::new('X').locked());
        }
        app.cursor = (5, 3);
        let mut brush = BuildingBrush::new("Inn", 6, 4);
        brush.on_confirm(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(6, 4)].ch, 'X');
    }
}
