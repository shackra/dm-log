use crate::app::App;
use crate::editor::brush::Brush;
use crate::map::MapType;

/// Paints a single terrain character onto a Region map cell.
pub struct TerrainBrush {
    pub ch: char,
    pub terrain: String,
}

impl TerrainBrush {
    pub fn new(ch: char, terrain: impl Into<String>) -> Self {
        TerrainBrush { ch, terrain: terrain.into() }
    }
}

impl Brush for TerrainBrush {
    fn name(&self) -> &str { "Terrain" }
    fn preview_char(&self) -> char { self.ch }

    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if map.map_type != MapType::Region {
                return;
            }
            if let Some(layer) = map.layer_mut(z) {
                let cell = crate::map::Cell::new(self.ch).with_terrain(self.terrain.clone());
                layer.cells.insert((col, row), cell);
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Places a City marker (`C`) on a Region map and creates a child City MapDef.
pub struct CityMarkerBrush {
    pub child_name: String,
    pub child_w: u16,
    pub child_h: u16,
}

impl CityMarkerBrush {
    pub fn new(child_name: impl Into<String>, child_w: u16, child_h: u16) -> Self {
        CityMarkerBrush {
            child_name: child_name.into(),
            child_w,
            child_h,
        }
    }
}

impl Brush for CityMarkerBrush {
    fn name(&self) -> &str { "City Marker" }
    fn preview_char(&self) -> char { 'C' }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        let parent_id = app.current_map().map(|m| m.id.clone()).unwrap_or_default();
        let child_id = uuid::Uuid::new_v4().to_string();

        // Place marker on parent map
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                let cell = crate::map::Cell::new('C').with_terrain("city");
                layer.cells.insert((col, row), cell);
            }
        }

        // Create child City MapDef
        let mut child = crate::map::MapDef::new(
            child_id.clone(),
            self.child_name.clone(),
            MapType::City,
            self.child_w,
            self.child_h,
        );
        child.parent = Some(parent_id);
        child.parent_x = Some(col);
        child.parent_y = Some(row);

        app.push_map(child);
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Places a Dungeon marker (`D`) on a Region map and creates a child Dungeon MapDef.
pub struct DungeonMarkerBrush {
    pub child_name: String,
    pub child_w: u16,
    pub child_h: u16,
}

impl DungeonMarkerBrush {
    pub fn new(child_name: impl Into<String>, child_w: u16, child_h: u16) -> Self {
        DungeonMarkerBrush {
            child_name: child_name.into(),
            child_w,
            child_h,
        }
    }
}

impl Brush for DungeonMarkerBrush {
    fn name(&self) -> &str { "Dungeon Marker" }
    fn preview_char(&self) -> char { 'D' }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        let parent_id = app.current_map().map(|m| m.id.clone()).unwrap_or_default();
        let child_id = uuid::Uuid::new_v4().to_string();

        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                let cell = crate::map::Cell::new('D').with_terrain("dungeon");
                layer.cells.insert((col, row), cell);
            }
        }

        let mut child = crate::map::MapDef::new(
            child_id,
            self.child_name.clone(),
            MapType::Dungeon,
            self.child_w,
            self.child_h,
        );
        child.parent = Some(parent_id);
        child.parent_x = Some(col);
        child.parent_y = Some(row);

        app.push_map(child);
    }

    fn cancel(&mut self, _app: &mut App) {}
}

/// Places a Town marker (`T`) on a Region map.
pub struct TownMarkerBrush;

impl Brush for TownMarkerBrush {
    fn name(&self) -> &str { "Town Marker" }
    fn preview_char(&self) -> char { 'T' }
    fn on_move(&mut self, _app: &mut App) {}
    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;
        if let Some(map) = app.current_map_mut() {
            if let Some(layer) = map.layer_mut(z) {
                let cell = crate::map::Cell::new('T').with_terrain("town");
                layer.cells.insert((col, row), cell);
            }
        }
    }
    fn cancel(&mut self, _app: &mut App) {}
}
