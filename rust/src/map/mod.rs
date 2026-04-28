use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod xml;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MapType {
    Region,
    Dungeon,
    City,
    Building,
}

impl Default for MapType {
    fn default() -> Self {
        MapType::Region
    }
}

impl std::fmt::Display for MapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapType::Region => write!(f, "region"),
            MapType::Dungeon => write!(f, "dungeon"),
            MapType::City => write!(f, "city"),
            MapType::Building => write!(f, "building"),
        }
    }
}

impl std::str::FromStr for MapType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "region" => Ok(MapType::Region),
            "dungeon" => Ok(MapType::Dungeon),
            "city" => Ok(MapType::City),
            "building" => Ok(MapType::Building),
            other => Err(format!("unknown map type: {}", other)),
        }
    }
}

/// A single grid cell in a map layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// The CP437-compatible Unicode character to display.
    pub ch: char,
    /// Terrain tag (e.g. "forest", "water", "floor").
    pub terrain: String,
    /// UUID of the keyed org-mode entry linked to this cell, if any.
    pub key_uuid: Option<String>,
    /// Locked cells (e.g. outer building walls) cannot be erased or overwritten.
    pub locked: bool,
    /// Name of the height zone this cell belongs to within the layer.
    /// None = uses the layer's default height.
    pub height_zone: Option<String>,
    /// Foreground color as a 256-color index (0 = default/use map-type accent).
    pub fg_color: u8,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Cell {
            ch,
            terrain: String::new(),
            key_uuid: None,
            locked: false,
            height_zone: None,
            fg_color: 0,
        }
    }

    pub fn locked(mut self) -> Self {
        self.locked = true;
        self
    }

    pub fn with_terrain(mut self, terrain: impl Into<String>) -> Self {
        self.terrain = terrain.into();
        self
    }

    pub fn with_color(mut self, fg_color: u8) -> Self {
        self.fg_color = fg_color;
        self
    }
}

/// A named height zone within a layer.
/// Cells belonging to this zone are at `layer.height_m + offset_m` meters above the ground.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightZone {
    pub name: String,
    /// Height offset in meters from the layer's base height.
    pub offset_m: f32,
}

/// A single altitude layer of a map.
///
/// `z` is a logical index (0 = ground, positive = above, negative = below).
/// `height_m` is the physical height of this layer relative to the **previous** layer
/// in meters (e.g. 3.0 for a typical floor, -2.5 for a basement).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Logical layer index (0 = ground, + = above, - = below).
    pub z: i32,
    /// Physical height in meters relative to the previous layer.
    pub height_m: f32,
    /// Named height zones within this layer with individual offsets.
    pub height_zones: HashMap<String, HeightZone>,
    /// Sparse map of (col, row) → Cell.
    pub cells: HashMap<(u16, u16), Cell>,
}

impl Layer {
    pub fn new(z: i32, height_m: f32) -> Self {
        Layer {
            z,
            height_m,
            height_zones: HashMap::new(),
            cells: HashMap::new(),
        }
    }

    /// Absolute height of a cell at this layer, given the cumulative base height
    /// from all previous layers.
    pub fn cell_abs_height(&self, pos: (u16, u16), cumulative_base: f32) -> f32 {
        let base = cumulative_base + self.height_m;
        if let Some(cell) = self.cells.get(&pos) {
            if let Some(zone_name) = &cell.height_zone {
                if let Some(zone) = self.height_zones.get(zone_name) {
                    return base + zone.offset_m;
                }
            }
        }
        base
    }
}

/// A building entity within a City/Village map.
/// The outer perimeter cells are locked and cannot be erased.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    /// UUID of the keyed org-mode entry for this building, if any.
    pub key_uuid: Option<String>,
    /// UUID of the child Building-type MapDef for interior editing, if any.
    pub interior_map_id: Option<String>,
}

impl Building {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> Self {
        Building {
            id: id.into(),
            name: name.into(),
            x,
            y,
            w,
            h,
            key_uuid: None,
            interior_map_id: None,
        }
    }

    /// Returns true if (col, row) is on the outer perimeter of this building.
    #[allow(dead_code)]
    pub fn is_perimeter(&self, col: u16, row: u16) -> bool {
        col == self.x || col == self.x + self.w - 1 || row == self.y || row == self.y + self.h - 1
    }

    /// Returns true if (col, row) is within or on the boundary of this building.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x + self.w && row >= self.y && row < self.y + self.h
    }
}

/// A complete map definition (one page of the editor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDef {
    pub id: String,
    pub name: String,
    pub map_type: MapType,
    pub width: u16,
    pub height: u16,
    pub layers: Vec<Layer>,
    pub buildings: Vec<Building>,
    /// UUID of the parent MapDef, if this is a sub-map.
    pub parent: Option<String>,
    pub parent_x: Option<u16>,
    pub parent_y: Option<u16>,
}

impl MapDef {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        map_type: MapType,
        width: u16,
        height: u16,
    ) -> Self {
        let ground_layer = Layer::new(0, 0.0);
        MapDef {
            id: id.into(),
            name: name.into(),
            map_type,
            width,
            height,
            layers: vec![ground_layer],
            buildings: Vec::new(),
            parent: None,
            parent_x: None,
            parent_y: None,
        }
    }

    /// Get the layer with the given z index, or None.
    pub fn layer(&self, z: i32) -> Option<&Layer> {
        self.layers.iter().find(|l| l.z == z)
    }

    /// Get a mutable reference to the layer with the given z index, or None.
    pub fn layer_mut(&mut self, z: i32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.z == z)
    }

    /// Cumulative base height (sum of height_m for all layers with z <= target_z).
    pub fn cumulative_height(&self, target_z: i32) -> f32 {
        let mut layers: Vec<&Layer> = self.layers.iter().filter(|l| l.z <= target_z).collect();
        layers.sort_by_key(|l| l.z);
        layers.iter().map(|l| l.height_m).sum()
    }

    /// Find a building that contains the given cell position.
    pub fn building_at(&self, col: u16, row: u16) -> Option<&Building> {
        self.buildings.iter().find(|b| b.contains(col, row))
    }

    /// Find a mutable building that contains the given cell position.
    #[allow(dead_code)]
    pub fn building_at_mut(&mut self, col: u16, row: u16) -> Option<&mut Building> {
        self.buildings.iter_mut().find(|b| b.contains(col, row))
    }
}

/// The root container for all maps in a campaign.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapFile {
    pub maps: Vec<MapDef>,
}

impl MapFile {
    pub fn new() -> Self {
        MapFile { maps: Vec::new() }
    }

    pub fn get(&self, id: &str) -> Option<&MapDef> {
        self.maps.iter().find(|m| m.id == id)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: &str) -> Option<&mut MapDef> {
        self.maps.iter_mut().find(|m| m.id == id)
    }

    pub fn push(&mut self, map: MapDef) {
        self.maps.push(map);
    }

    /// Upsert a MapDef by id (replace if exists, append if new).
    pub fn upsert(&mut self, map: MapDef) {
        if let Some(existing) = self.maps.iter_mut().find(|m| m.id == map.id) {
            *existing = map;
        } else {
            self.maps.push(map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_perimeter() {
        let b = Building::new("id", "Tavern", 2, 3, 5, 4);
        // corners
        assert!(b.is_perimeter(2, 3));
        assert!(b.is_perimeter(6, 3));
        assert!(b.is_perimeter(2, 6));
        assert!(b.is_perimeter(6, 6));
        // interior
        assert!(!b.is_perimeter(4, 4));
        // outside
        assert!(!b.contains(1, 3));
    }

    #[test]
    fn cumulative_height() {
        let mut map = MapDef::new("id", "Test", MapType::Dungeon, 40, 20);
        map.layers[0].height_m = 0.0; // ground
        let l1 = Layer::new(1, 3.5);
        map.layers.push(l1.clone());
        let l2 = Layer::new(2, 2.8);
        map.layers.push(l2);
        assert!((map.cumulative_height(0) - 0.0).abs() < 0.001);
        assert!((map.cumulative_height(1) - 3.5).abs() < 0.001);
        assert!((map.cumulative_height(2) - 6.3).abs() < 0.01);
    }

    #[test]
    fn cell_abs_height_with_zone() {
        let mut layer = Layer::new(1, 3.0);
        layer.height_zones.insert(
            "pit".to_string(),
            HeightZone {
                name: "pit".to_string(),
                offset_m: -1.5,
            },
        );
        let mut cell = Cell::new('.');
        cell.height_zone = Some("pit".to_string());
        layer.cells.insert((5, 5), cell);
        // cumulative_base from previous layer = 0.0
        assert!((layer.cell_abs_height((5, 5), 0.0) - 1.5).abs() < 0.001);
        // cell without zone
        layer.cells.insert((6, 6), Cell::new('.'));
        assert!((layer.cell_abs_height((6, 6), 0.0) - 3.0).abs() < 0.001);
    }
}
