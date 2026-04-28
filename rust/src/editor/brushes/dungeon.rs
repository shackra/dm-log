use crate::app::App;
use crate::editor::brush::Brush;
use crate::map::{Cell, MapType};

/// Draws a rectangular room with `#` walls and `.` floor.
/// Resize with `+`/`-` before confirming.
pub struct RoomBrush {
    pub w: u16,
    pub h: u16,
}

impl RoomBrush {
    pub fn new(w: u16, h: u16) -> Self {
        RoomBrush {
            w: w.max(3),
            h: h.max(3),
        }
    }

    #[allow(dead_code)]
    pub fn grow(&mut self) {
        self.w += 1;
        self.h += 1;
    }

    #[allow(dead_code)]
    pub fn shrink(&mut self) {
        self.w = (self.w - 1).max(3);
        self.h = (self.h - 1).max(3);
    }

    /// Cells that would be placed at origin (col, row).
    fn cells_at(&self, col: u16, row: u16) -> Vec<((u16, u16), Cell)> {
        let mut out = Vec::new();
        for dy in 0..self.h {
            for dx in 0..self.w {
                let r = row + dy;
                let c = col + dx;
                let on_wall = dy == 0 || dy == self.h - 1 || dx == 0 || dx == self.w - 1;
                let cell = if on_wall {
                    Cell::new('#').with_terrain("wall")
                } else {
                    Cell::new('.').with_terrain("floor")
                };
                out.push(((c, r), cell));
            }
        }
        out
    }
}

impl Brush for RoomBrush {
    fn name(&self) -> &str {
        "Room"
    }
    fn preview_char(&self) -> char {
        '#'
    }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if map.map_type != MapType::Dungeon && map.map_type != MapType::Building {
                return;
            }
            if let Some(layer) = map.layer_mut(z) {
                for (pos, cell) in self.cells_at(col, row) {
                    // Don't overwrite locked cells
                    if layer.cells.get(&pos).map(|c| c.locked).unwrap_or(false) {
                        continue;
                    }
                    layer.cells.insert(pos, cell);
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Draws a corridor (`.` with `#` side walls), horizontal or vertical.
pub struct CorridorBrush {
    pub horizontal: bool,
    pub length: u16,
}

impl CorridorBrush {
    pub fn new(horizontal: bool, length: u16) -> Self {
        CorridorBrush {
            horizontal,
            length: length.max(1),
        }
    }
}

impl Brush for CorridorBrush {
    fn name(&self) -> &str {
        "Corridor"
    }
    fn preview_char(&self) -> char {
        '.'
    }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if self.horizontal {
                    // wall row above + below, floor in middle
                    for dx in 0..self.length {
                        let c = col + dx;
                        for dr in [0u16, 1, 2] {
                            let r = row + dr;
                            let is_wall = dr == 0 || dr == 2;
                            let cell = if is_wall {
                                Cell::new('#').with_terrain("wall")
                            } else {
                                Cell::new('.').with_terrain("floor")
                            };
                            if !layer.cells.get(&(c, r)).map(|c| c.locked).unwrap_or(false) {
                                layer.cells.insert((c, r), cell);
                            }
                        }
                    }
                } else {
                    // vertical
                    for dy in 0..self.length {
                        let r = row + dy;
                        for dc in [0u16, 1, 2] {
                            let c = col + dc;
                            let is_wall = dc == 0 || dc == 2;
                            let cell = if is_wall {
                                Cell::new('#').with_terrain("wall")
                            } else {
                                Cell::new('.').with_terrain("floor")
                            };
                            if !layer.cells.get(&(c, r)).map(|c| c.locked).unwrap_or(false) {
                                layer.cells.insert((c, r), cell);
                            }
                        }
                    }
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Places a door character (`+` closed, `/` open) snapped to a wall cell.
pub struct DoorBrush {
    pub open: bool,
}

impl DoorBrush {
    pub fn new(open: bool) -> Self {
        DoorBrush { open }
    }
}

impl Brush for DoorBrush {
    fn name(&self) -> &str {
        if self.open {
            "Door (open)"
        } else {
            "Door (closed)"
        }
    }
    fn preview_char(&self) -> char {
        if self.open { '/' } else { '+' }
    }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        let ch = if self.open { '/' } else { '+' };
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if !layer
                    .cells
                    .get(&(col, row))
                    .map(|c| c.locked)
                    .unwrap_or(false)
                {
                    layer
                        .cells
                        .insert((col, row), Cell::new(ch).with_terrain("door"));
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Places a staircase (`<` up, `>` down).
pub struct StairsBrush {
    pub up: bool,
    /// Target layer z-index.
    pub target_z: i32,
}

impl StairsBrush {
    pub fn new(up: bool, target_z: i32) -> Self {
        StairsBrush { up, target_z }
    }
}

impl Brush for StairsBrush {
    fn name(&self) -> &str {
        if self.up {
            "Stairs (up)"
        } else {
            "Stairs (down)"
        }
    }
    fn preview_char(&self) -> char {
        if self.up { '<' } else { '>' }
    }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        let ch = if self.up { '<' } else { '>' };
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                if !layer
                    .cells
                    .get(&(col, row))
                    .map(|c| c.locked)
                    .unwrap_or(false)
                {
                    layer
                        .cells
                        .insert((col, row), Cell::new(ch).with_terrain("stairs"));
                }
            }
        }
        let dir = if self.up { "up" } else { "down" };
        app.set_status(format!("Stairs ({dir}) → layer {}", self.target_z));
    }

    fn cancel(&mut self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::map::{Layer, MapDef, MapType};
    use std::path::PathBuf;

    fn dungeon_app() -> App {
        let mut map = MapDef::new("d1", "Dungeon", MapType::Dungeon, 40, 20);
        map.layers[0] = Layer::new(0, 0.0);
        App::new(PathBuf::from("/tmp"), Some(map))
    }

    #[test]
    fn room_brush_places_walls_and_floor() {
        let mut app = dungeon_app();
        app.cursor = (2, 2);
        let mut brush = RoomBrush::new(5, 4);
        brush.on_confirm(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(2, 2)].ch, '#');
        assert_eq!(layer.cells[&(6, 5)].ch, '#');
        assert_eq!(layer.cells[&(3, 3)].ch, '.');
        assert_eq!(layer.cells[&(4, 4)].ch, '.');
    }

    #[test]
    fn room_brush_respects_locked_cells() {
        let mut app = dungeon_app();
        {
            let map = app.current_map_mut().unwrap();
            let layer = map.layer_mut(0).unwrap();
            layer
                .cells
                .insert((3, 3), crate::map::Cell::new('X').locked());
        }
        app.cursor = (2, 2);
        let mut brush = RoomBrush::new(5, 4);
        brush.on_confirm(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(3, 3)].ch, 'X');
    }

    #[test]
    fn door_brush_places_char() {
        let mut app = dungeon_app();
        app.cursor = (5, 5);
        let mut brush = DoorBrush::new(false);
        brush.on_confirm(&mut app);
        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(5, 5)].ch, '+');
    }

    #[test]
    fn stairs_brush_places_char_and_sets_status() {
        let mut app = dungeon_app();
        app.cursor = (3, 3);
        let mut brush = StairsBrush::new(true, 1);
        brush.on_confirm(&mut app);
        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(3, 3)].ch, '<');
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn room_brush_resize() {
        let mut brush = RoomBrush::new(5, 5);
        brush.grow();
        assert_eq!(brush.w, 6);
        brush.shrink();
        brush.shrink();
        assert_eq!(brush.w, 4);
        brush.w = 3;
        brush.shrink();
        assert_eq!(brush.w, 3);
    }
}
