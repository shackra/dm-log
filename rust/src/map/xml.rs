/// XML serialization/deserialization for MapFile.
///
/// Format:
/// ```xml
/// <maps>
///   <map id="..." name="..." type="region" width="80" height="40"
///        parent="" parent_x="" parent_y="">
///     <layer z="0" height_m="0.0">
///       <height_zone name="pit" offset_m="-1.5"/>
///       <cell x="5" y="3" ch="." terrain="floor" locked="false"
///             key_uuid="" height_zone=""/>
///     </layer>
///     <building id="..." name="Tavern" x="5" y="3" w="8" h="6"
///               key_uuid="" interior_map_id=""/>
///   </map>
/// </maps>
/// ```
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use super::{Building, Cell, HeightZone, Layer, MapDef, MapFile, MapType};

// ──────────────────────────────────────────────────────────────────────────────
// Save
// ──────────────────────────────────────────────────────────────────────────────

pub fn save(path: &Path, file: &MapFile) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let mut w = Writer::new_with_indent(&mut buf, b' ', 2);

    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let maps_start = BytesStart::new("maps");
    w.write_event(Event::Start(maps_start))?;

    for map in &file.maps {
        write_map(&mut w, map)?;
    }

    w.write_event(Event::End(BytesEnd::new("maps")))?;

    std::fs::write(path, &buf)?;
    Ok(())
}

fn write_map<W: Write>(w: &mut Writer<W>, map: &MapDef) -> Result<(), Box<dyn std::error::Error>> {
    let mut start = BytesStart::new("map");
    start.push_attribute(("id", map.id.as_str()));
    start.push_attribute(("name", map.name.as_str()));
    start.push_attribute(("type", map.map_type.to_string().as_str()));
    start.push_attribute(("width", map.width.to_string().as_str()));
    start.push_attribute(("height", map.height.to_string().as_str()));
    if let Some(p) = &map.parent {
        start.push_attribute(("parent", p.as_str()));
    }
    if let Some(px) = map.parent_x {
        start.push_attribute(("parent_x", px.to_string().as_str()));
    }
    if let Some(py) = map.parent_y {
        start.push_attribute(("parent_y", py.to_string().as_str()));
    }
    w.write_event(Event::Start(start))?;

    // Layers sorted by z
    let mut layers: Vec<&Layer> = map.layers.iter().collect();
    layers.sort_by_key(|l| l.z);
    for layer in layers {
        write_layer(w, layer)?;
    }

    // Buildings
    for building in &map.buildings {
        write_building(w, building)?;
    }

    w.write_event(Event::End(BytesEnd::new("map")))?;
    Ok(())
}

fn write_layer<W: Write>(
    w: &mut Writer<W>,
    layer: &Layer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut start = BytesStart::new("layer");
    start.push_attribute(("z", layer.z.to_string().as_str()));
    start.push_attribute(("height_m", layer.height_m.to_string().as_str()));
    w.write_event(Event::Start(start))?;

    // Height zones
    let mut zones: Vec<&HeightZone> = layer.height_zones.values().collect();
    zones.sort_by(|a, b| a.name.cmp(&b.name));
    for zone in zones {
        let mut ze = BytesStart::new("height_zone");
        ze.push_attribute(("name", zone.name.as_str()));
        ze.push_attribute(("offset_m", zone.offset_m.to_string().as_str()));
        w.write_event(Event::Empty(ze))?;
    }

    // Cells sorted for deterministic output
    let mut cells: Vec<(&(u16, u16), &Cell)> = layer.cells.iter().collect();
    cells.sort_by_key(|(pos, _)| **pos);
    for ((x, y), cell) in cells {
        write_cell(w, *x, *y, cell)?;
    }

    w.write_event(Event::End(BytesEnd::new("layer")))?;
    Ok(())
}

fn write_cell<W: Write>(
    w: &mut Writer<W>,
    x: u16,
    y: u16,
    cell: &Cell,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ce = BytesStart::new("cell");
    ce.push_attribute(("x", x.to_string().as_str()));
    ce.push_attribute(("y", y.to_string().as_str()));
    // Encode char as Unicode codepoint hex to avoid XML special-char issues
    ce.push_attribute(("ch", format!("U+{:04X}", cell.ch as u32).as_str()));
    ce.push_attribute(("terrain", cell.terrain.as_str()));
    ce.push_attribute(("locked", cell.locked.to_string().as_str()));
    if let Some(ku) = &cell.key_uuid {
        ce.push_attribute(("key_uuid", ku.as_str()));
    }
    if let Some(hz) = &cell.height_zone {
        ce.push_attribute(("height_zone", hz.as_str()));
    }
    w.write_event(Event::Empty(ce))?;
    Ok(())
}

fn write_building<W: Write>(
    w: &mut Writer<W>,
    b: &Building,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut be = BytesStart::new("building");
    be.push_attribute(("id", b.id.as_str()));
    be.push_attribute(("name", b.name.as_str()));
    be.push_attribute(("x", b.x.to_string().as_str()));
    be.push_attribute(("y", b.y.to_string().as_str()));
    be.push_attribute(("w", b.w.to_string().as_str()));
    be.push_attribute(("h", b.h.to_string().as_str()));
    if let Some(ku) = &b.key_uuid {
        be.push_attribute(("key_uuid", ku.as_str()));
    }
    if let Some(im) = &b.interior_map_id {
        be.push_attribute(("interior_map_id", im.as_str()));
    }
    w.write_event(Event::Empty(be))?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Load
// ──────────────────────────────────────────────────────────────────────────────

pub fn load(path: &Path) -> Result<MapFile, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(MapFile::new());
    }
    let content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut map_file = MapFile::new();
    let mut buf = Vec::new();

    // State machine
    let mut current_map: Option<MapDef> = None;
    let mut current_layer: Option<Layer> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"map" => {
                    let attrs = parse_attrs(e);
                    let map_type: MapType = attrs
                        .get("type")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(MapType::Region);
                    let map = MapDef {
                        id: attrs.get("id").cloned().unwrap_or_default(),
                        name: attrs.get("name").cloned().unwrap_or_default(),
                        map_type,
                        width: attrs
                            .get("width")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(80),
                        height: attrs
                            .get("height")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(40),
                        layers: Vec::new(),
                        buildings: Vec::new(),
                        parent: attrs.get("parent").cloned(),
                        parent_x: attrs.get("parent_x").and_then(|s| s.parse().ok()),
                        parent_y: attrs.get("parent_y").and_then(|s| s.parse().ok()),
                    };
                    current_map = Some(map);
                }
                b"layer" => {
                    let attrs = parse_attrs(e);
                    let z: i32 = attrs.get("z").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let height_m: f32 = attrs
                        .get("height_m")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    current_layer = Some(Layer::new(z, height_m));
                }
                b"height_zone" => {
                    if let Some(layer) = current_layer.as_mut() {
                        let attrs = parse_attrs(e);
                        let name = attrs.get("name").cloned().unwrap_or_default();
                        let offset_m: f32 = attrs
                            .get("offset_m")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        layer
                            .height_zones
                            .insert(name.clone(), HeightZone { name, offset_m });
                    }
                }
                b"cell" => {
                    if let Some(layer) = current_layer.as_mut() {
                        let attrs = parse_attrs(e);
                        let x: u16 = attrs.get("x").and_then(|s| s.parse().ok()).unwrap_or(0);
                        let y: u16 = attrs.get("y").and_then(|s| s.parse().ok()).unwrap_or(0);
                        let ch = parse_char(attrs.get("ch").map(|s| s.as_str()).unwrap_or("?"));
                        let mut cell = Cell::new(ch);
                        cell.terrain = attrs.get("terrain").cloned().unwrap_or_default();
                        cell.locked = attrs.get("locked").map(|s| s == "true").unwrap_or(false);
                        cell.key_uuid = attrs.get("key_uuid").cloned();
                        cell.height_zone = attrs.get("height_zone").cloned();
                        layer.cells.insert((x, y), cell);
                    }
                }
                b"building" => {
                    if let Some(map) = current_map.as_mut() {
                        let attrs = parse_attrs(e);
                        let mut b = Building::new(
                            attrs.get("id").cloned().unwrap_or_default(),
                            attrs.get("name").cloned().unwrap_or_default(),
                            attrs.get("x").and_then(|s| s.parse().ok()).unwrap_or(0),
                            attrs.get("y").and_then(|s| s.parse().ok()).unwrap_or(0),
                            attrs.get("w").and_then(|s| s.parse().ok()).unwrap_or(1),
                            attrs.get("h").and_then(|s| s.parse().ok()).unwrap_or(1),
                        );
                        b.key_uuid = attrs.get("key_uuid").cloned();
                        b.interior_map_id = attrs.get("interior_map_id").cloned();
                        map.buildings.push(b);
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"layer" => {
                    if let (Some(layer), Some(map)) = (current_layer.take(), current_map.as_mut()) {
                        map.layers.push(layer);
                    }
                }
                b"map" => {
                    if let Some(map) = current_map.take() {
                        map_file.maps.push(map);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(map_file)
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn parse_attrs(e: &BytesStart) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(attr.value.as_ref()).to_string();
        map.insert(key, val);
    }
    map
}

/// Parse a character stored as "U+XXXX" hex codepoint or a raw single char.
fn parse_char(s: &str) -> char {
    if let Some(hex) = s.strip_prefix("U+") {
        if let Ok(n) = u32::from_str_radix(hex, 16) {
            if let Some(c) = char::from_u32(n) {
                return c;
            }
        }
    }
    s.chars().next().unwrap_or('?')
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map_file() -> MapFile {
        let mut mf = MapFile::new();

        let mut map = MapDef::new("map-001", "The Realm", MapType::Region, 80, 40);
        {
            let layer = &mut map.layers[0];
            layer.height_m = 0.0;
            layer.height_zones.insert(
                "valley".to_string(),
                HeightZone {
                    name: "valley".to_string(),
                    offset_m: -10.0,
                },
            );
            layer.cells.insert(
                (10, 5),
                Cell {
                    ch: '♠',
                    terrain: "forest".to_string(),
                    key_uuid: Some("uuid-abc".to_string()),
                    locked: false,
                    height_zone: Some("valley".to_string()),
                },
            );
        }

        let mut city_map = MapDef::new("map-002", "Ironhaven", MapType::City, 60, 30);
        city_map.parent = Some("map-001".to_string());
        city_map.parent_x = Some(10);
        city_map.parent_y = Some(5);
        city_map.buildings.push(Building {
            id: "bld-001".to_string(),
            name: "Tavern".to_string(),
            x: 5,
            y: 3,
            w: 8,
            h: 6,
            key_uuid: Some("uuid-tavern".to_string()),
            interior_map_id: None,
        });

        mf.push(map);
        mf.push(city_map);
        mf
    }

    #[test]
    fn xml_round_trip() {
        let original = sample_map_file();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("maps.xml");

        save(&path, &original).expect("save failed");
        let loaded = load(&path).expect("load failed");

        assert_eq!(loaded.maps.len(), 2);

        let realm = loaded.get("map-001").unwrap();
        assert_eq!(realm.name, "The Realm");
        assert_eq!(realm.map_type, MapType::Region);
        assert_eq!(realm.width, 80);
        assert_eq!(realm.height, 40);

        let layer0 = realm.layer(0).unwrap();
        assert_eq!(layer0.height_zones.len(), 1);
        let valley = &layer0.height_zones["valley"];
        assert!((valley.offset_m - (-10.0)).abs() < 0.001);

        let cell = layer0.cells.get(&(10, 5)).unwrap();
        assert_eq!(cell.ch, '♠');
        assert_eq!(cell.terrain, "forest");
        assert_eq!(cell.key_uuid.as_deref(), Some("uuid-abc"));
        assert_eq!(cell.height_zone.as_deref(), Some("valley"));

        let city = loaded.get("map-002").unwrap();
        assert_eq!(city.parent.as_deref(), Some("map-001"));
        assert_eq!(city.parent_x, Some(10));
        assert_eq!(city.buildings.len(), 1);
        assert_eq!(city.buildings[0].name, "Tavern");
        assert_eq!(city.buildings[0].key_uuid.as_deref(), Some("uuid-tavern"));
    }

    #[test]
    fn load_nonexistent_returns_empty() {
        let path = std::path::Path::new("/tmp/nonexistent-mazaforja-test.xml");
        let mf = load(path).unwrap();
        assert!(mf.maps.is_empty());
    }

    #[test]
    fn parse_char_codepoint() {
        assert_eq!(parse_char("U+2660"), '♠');
        assert_eq!(parse_char("U+002E"), '.');
        assert_eq!(parse_char("#"), '#');
    }
}
