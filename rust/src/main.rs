mod app;
mod editor;
mod map;
mod ui;

use std::path::PathBuf;

use clap::Parser;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};

use app::{App, EditorMode};
use map::{Layer, MapDef, MapFile, MapType};
use ui::{
    brush_picker::{BrushEntry, BrushPickerWidget},
    canvas::Canvas,
    layers::LayerPanel,
    palette::{Palette, PaletteWidget},
    status::StatusBar,
};

// ──────────────────────────────────────────────────────────────────────────────
// CLI args
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "mazaforja", about = "dm-log ASCII map editor")]
struct Cli {
    /// Campaign directory (contains maps.xml, map.org, etc.)
    #[arg(long, value_name = "DIR")]
    campaign_dir: PathBuf,

    /// Load a specific map by UUID at startup (optional)
    #[arg(long, value_name = "UUID")]
    map_id: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Entry point
// ──────────────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load or create map file
    let maps_xml = cli.campaign_dir.join("maps.xml");
    let mut map_file = map::xml::load(&maps_xml)?;

    // Resolve initial map
    let initial_map = if let Some(ref id) = cli.map_id {
        map_file.get(id).cloned()
    } else {
        map_file.maps.first().cloned()
    };

    // If no maps exist yet, create a default Region
    let initial_map = initial_map.unwrap_or_else(|| {
        let m = MapDef::new(
            uuid::Uuid::new_v4().to_string(),
            "Untitled Region",
            MapType::Region,
            80,
            40,
        );
        map_file.push(m.clone());
        m
    });

    let mut app = App::new(cli.campaign_dir.clone(), Some(initial_map));
    let mut palette = Palette::default();

    // Brush-picker entries vary by map type; we'll rebuild on demand.
    let mut brush_picker_selected: usize = 0;
    let mut brush_picker_entries: Vec<BrushEntry> = Vec::new();

    // Terminal init — install panic hook to restore terminal even on crash
    let mut terminal = ratatui::init();

    let result = run(
        &mut terminal,
        &mut app,
        &mut map_file,
        &mut palette,
        &mut brush_picker_selected,
        &mut brush_picker_entries,
    );

    ratatui::restore();
    result
}

// ──────────────────────────────────────────────────────────────────────────────
// Main render + event loop
// ──────────────────────────────────────────────────────────────────────────────

fn run(
    terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    map_file: &mut MapFile,
    palette: &mut Palette,
    brush_picker_selected: &mut usize,
    brush_picker_entries: &mut Vec<BrushEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Clear one-shot status
        let saved_status = app.status_msg.take();

        terminal.draw(|frame| {
            let size = frame.area();

            // Restore status for this frame
            app.status_msg = saved_status.clone();

            // Layout: optional layer panel (left 12 cols) | canvas | right
            let (canvas_area, layer_area) = if app.layer_panel_open {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(1), Constraint::Length(14)])
                    .split(size);
                (chunks[0], Some(chunks[1]))
            } else {
                (size, None)
            };

            // Status bar at bottom (2 rows)
            let (canvas_area, status_area) = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(2)])
                    .split(canvas_area);
                (chunks[0], chunks[1])
            };

            // Auto-scroll
            app.scroll_to_cursor(canvas_area.width, canvas_area.height);

            // Render canvas
            frame.render_widget(Canvas::new(app), canvas_area);

            // Render status bar
            frame.render_widget(StatusBar::new(app), status_area);

            // Render layer panel
            if let Some(la) = layer_area {
                frame.render_widget(LayerPanel::new(app), la);
            }

            // Render palette popup (centered)
            if app.palette_open {
                let popup = centered_rect(40, 60, size);
                frame.render_widget(
                    ratatui::widgets::Clear,
                    popup,
                );
                frame.render_widget(PaletteWidget { palette }, popup);
            }

            // Render brush picker popup (centered)
            if app.mode == EditorMode::BrushPicker {
                let popup = centered_rect(36, 50, size);
                frame.render_widget(ratatui::widgets::Clear, popup);
                frame.render_widget(
                    BrushPickerWidget {
                        entries: brush_picker_entries,
                        selected: *brush_picker_selected,
                    },
                    popup,
                );
            }

            // Clear status after render so it doesn't persist multiple frames
            app.status_msg = None;
        })?;

        // Event handling (blocking poll)
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Palette popup has its own key handling
            if app.palette_open {
                handle_palette_key(key.code, app, palette);
                continue;
            }

            // Brush picker popup
            if app.mode == EditorMode::BrushPicker {
                handle_brush_picker_key(
                    key.code,
                    app,
                    map_file,
                    brush_picker_selected,
                    brush_picker_entries,
                );
                continue;
            }

            match app.mode {
                EditorMode::Normal => handle_normal_key(
                    key.code,
                    key.modifiers,
                    app,
                    map_file,
                    brush_picker_selected,
                    brush_picker_entries,
                ),
                EditorMode::Brushing => handle_brushing_key(key.code, app, map_file),
                EditorMode::ZonePaint => handle_zone_paint_key(key.code, app, map_file),
                _ => {}
            }

            if app.should_quit {
                save_all(app, map_file);
                break;
            }
        }
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Key handlers
// ──────────────────────────────────────────────────────────────────────────────

fn handle_normal_key(
    code: KeyCode,
    _mods: KeyModifiers,
    app: &mut App,
    map_file: &mut MapFile,
    brush_picker_selected: &mut usize,
    brush_picker_entries: &mut Vec<BrushEntry>,
) {
    match code {
        // Movement
        KeyCode::Char('h') | KeyCode::Left  => app.move_cursor(-1, 0),
        KeyCode::Char('j') | KeyCode::Down  => app.move_cursor(0, 1),
        KeyCode::Char('k') | KeyCode::Up    => app.move_cursor(0, -1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1, 0),

        // Layer navigation
        KeyCode::Char('[') => app.prev_layer(),
        KeyCode::Char(']') => app.next_layer(),

        // Brush picker
        KeyCode::Char('b') => {
            *brush_picker_entries = build_brush_entries(app);
            *brush_picker_selected = 0;
            app.mode = EditorMode::BrushPicker;
        }

        // Palette
        KeyCode::Char('p') => {
            app.palette_open = !app.palette_open;
        }

        // Layer panel toggle
        KeyCode::Char('L') => {
            app.layer_panel_open = !app.layer_panel_open;
        }

        // Zone paint mode
        KeyCode::Char('Z') => {
            app.mode = EditorMode::ZonePaint;
            app.set_status("ZONE PAINT: move cursor to flood-fill area, Enter=fill, Esc=cancel");
        }

        // Keying
        KeyCode::Char('K') => {
            editor::keying::key_current_cell(app);
            // Sync map stack back into map_file
            sync_stack_to_file(app, map_file);
        }

        // Enter = activate brush / drill into child
        KeyCode::Enter => {
            try_drill_down(app, map_file);
        }

        // Esc = pop map stack / cancel
        KeyCode::Esc => {
            if !app.pop_map() {
                app.set_status("Press 'q' to quit.");
            }
        }

        // Quit + save
        KeyCode::Char('q') => {
            app.should_quit = true;
        }

        // Add layer above current
        KeyCode::Char('+') => {
            let new_z = app.current_layer + 1;
            if let Some(map) = app.current_map_mut() {
                if map.layer(new_z).is_none() {
                    map.layers.push(Layer::new(new_z, 3.0));
                    app.set_status(format!("Added layer z={new_z} (+3.0m)"));
                } else {
                    app.set_status("Layer already exists.");
                }
            }
            sync_stack_to_file(app, map_file);
        }

        // Add layer below
        KeyCode::Char('-') => {
            let new_z = app.current_layer - 1;
            if let Some(map) = app.current_map_mut() {
                if map.layer(new_z).is_none() {
                    map.layers.push(Layer::new(new_z, -3.0));
                    app.set_status(format!("Added layer z={new_z} (-3.0m)"));
                } else {
                    app.set_status("Layer already exists.");
                }
            }
            sync_stack_to_file(app, map_file);
        }

        _ => {}
    }
}

/// Swap brush out, call `f`, swap back. Avoids double-borrow of `app`.
fn with_brush<F>(app: &mut App, f: F)
where
    F: FnOnce(&mut dyn editor::brush::Brush, &mut App),
{
    let mut brush = std::mem::replace(&mut app.active_brush, Box::new(editor::brush::NullBrush));
    f(brush.as_mut(), app);
    app.active_brush = brush;
}

fn handle_brushing_key(code: KeyCode, app: &mut App, map_file: &mut MapFile) {
    match code {
        KeyCode::Char('h') | KeyCode::Left  => { app.move_cursor(-1, 0); with_brush(app, |b, a| b.on_move(a)); }
        KeyCode::Char('j') | KeyCode::Down  => { app.move_cursor(0, 1);  with_brush(app, |b, a| b.on_move(a)); }
        KeyCode::Char('k') | KeyCode::Up    => { app.move_cursor(0, -1); with_brush(app, |b, a| b.on_move(a)); }
        KeyCode::Char('l') | KeyCode::Right => { app.move_cursor(1, 0);  with_brush(app, |b, a| b.on_move(a)); }
        KeyCode::Enter | KeyCode::Char(' ') => {
            with_brush(app, |b, a| b.on_confirm(a));
            sync_stack_to_file(app, map_file);
            save_all(app, map_file);
        }
        KeyCode::Esc => {
            with_brush(app, |b, a| b.cancel(a));
            app.mode = EditorMode::Normal;
        }
        _ => {}
    }
}

fn handle_palette_key(code: KeyCode, app: &mut App, palette: &mut Palette) {
    match code {
        KeyCode::Char('h') | KeyCode::Left  => palette.move_left(),
        KeyCode::Char('j') | KeyCode::Down  => palette.move_down(),
        KeyCode::Char('k') | KeyCode::Up    => palette.move_up(),
        KeyCode::Char('l') | KeyCode::Right => palette.move_right(),
        KeyCode::Tab => palette.next_category(),
        KeyCode::BackTab => palette.prev_category(),
        KeyCode::Enter => {
            if let Some(ch) = palette.selected_char() {
                app.active_brush = Box::new(
                    editor::brushes::region::TerrainBrush::new(ch, "")
                );
                app.mode = EditorMode::Brushing;
            }
            app.palette_open = false;
        }
        KeyCode::Esc => {
            app.palette_open = false;
        }
        _ => {}
    }
}

fn handle_brush_picker_key(
    code: KeyCode,
    app: &mut App,
    _map_file: &mut MapFile,
    selected: &mut usize,
    entries: &mut Vec<BrushEntry>,
) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !entries.is_empty() {
                *selected = (*selected + 1) % entries.len();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if !entries.is_empty() {
                *selected = selected.wrapping_sub(1).min(entries.len() - 1);
            }
        }
        KeyCode::Enter => {
            activate_brush_by_index(app, *selected);
            app.mode = EditorMode::Brushing;
        }
        KeyCode::Esc => {
            app.mode = EditorMode::Normal;
        }
        _ => {}
    }
}

fn handle_zone_paint_key(code: KeyCode, app: &mut App, map_file: &mut MapFile) {
    match code {
        KeyCode::Char('h') | KeyCode::Left  => app.move_cursor(-1, 0),
        KeyCode::Char('j') | KeyCode::Down  => app.move_cursor(0, 1),
        KeyCode::Char('k') | KeyCode::Up    => app.move_cursor(0, -1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1, 0),
        KeyCode::Enter => {
            zone_flood_fill(app);
            sync_stack_to_file(app, map_file);
            save_all(app, map_file);
            app.mode = EditorMode::Normal;
        }
        KeyCode::Esc => {
            app.mode = EditorMode::Normal;
        }
        _ => {}
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Zone flood-fill
// ──────────────────────────────────────────────────────────────────────────────

fn zone_flood_fill(app: &mut App) {
    let (start_col, start_row) = app.cursor;
    let z = app.current_layer;
    let zone_name = format!("zone-{}", uuid::Uuid::new_v4().simple());

    let target_ch = if let Some(map) = app.current_map() {
        if let Some(layer) = map.layer(z) {
            layer.cells.get(&(start_col, start_row)).map(|c| c.ch).unwrap_or(' ')
        } else { return; }
    } else { return; };

    let map_w = app.current_map().map(|m| m.width).unwrap_or(80);
    let map_h = app.current_map().map(|m| m.height).unwrap_or(40);

    // Collect cells to update via BFS
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((start_col, start_row));

    while let Some((col, row)) = queue.pop_front() {
        if visited.contains(&(col, row)) { continue; }
        if col >= map_w || row >= map_h { continue; }

        let matches = if let Some(map) = app.current_map() {
            if let Some(layer) = map.layer(z) {
                layer.cells.get(&(col, row)).map(|c| c.ch == target_ch).unwrap_or(target_ch == ' ')
            } else { false }
        } else { false };

        if !matches { continue; }
        visited.insert((col, row));

        if col > 0 { queue.push_back((col - 1, row)); }
        if col + 1 < map_w { queue.push_back((col + 1, row)); }
        if row > 0 { queue.push_back((col, row - 1)); }
        if row + 1 < map_h { queue.push_back((col, row + 1)); }
    }

    // Apply zone assignments
    if let Some(map) = app.current_map_mut() {
        if let Some(layer) = map.layer_mut(z) {
            for pos in &visited {
                let cell = layer.cells.entry(*pos).or_insert_with(|| map::Cell::new(target_ch));
                cell.height_zone = Some(zone_name.clone());
            }
            // Register zone with default offset 0.0
            layer.height_zones.insert(
                zone_name.clone(),
                map::HeightZone { name: zone_name.clone(), offset_m: 0.0 },
            );
        }
    }

    app.set_status(format!("Zone '{zone_name}' created ({} cells). Use Zh to set height.", visited.len()));
}

// ──────────────────────────────────────────────────────────────────────────────
// Drill-down navigation
// ──────────────────────────────────────────────────────────────────────────────

fn try_drill_down(app: &mut App, map_file: &mut MapFile) {
    let (col, row) = app.cursor;
    let z = app.current_layer;

    // Look for a child map linked from the cell at cursor
    let child_id = if let Some(map) = app.current_map() {
        if let Some(layer) = map.layer(z) {
            layer.cells.get(&(col, row)).and_then(|c| c.key_uuid.clone())
        } else { None }
    } else { None };

    if let Some(ref id) = child_id {
        if let Some(child) = map_file.get(id).cloned() {
            app.push_map(child);
            return;
        }
    }

    // City map: Enter on building cell → push building interior if exists
    let interior_id = if let Some(map) = app.current_map() {
        if map.map_type == MapType::City {
            map.building_at(col, row)
                .and_then(|b| b.interior_map_id.clone())
        } else { None }
    } else { None };

    if let Some(ref id) = interior_id {
        if let Some(interior) = map_file.get(id).cloned() {
            app.push_map(interior);
            return;
        }
    }

    // Nothing to drill into
    app.set_status("Nothing to drill into here.");
}

// ──────────────────────────────────────────────────────────────────────────────
// Brush registry
// ──────────────────────────────────────────────────────────────────────────────

fn build_brush_entries(app: &App) -> Vec<BrushEntry> {
    let map_type = app.current_map().map(|m| m.map_type.clone());
    match map_type {
        Some(MapType::Region) => vec![
            BrushEntry { name: "Terrain".into(), preview_char: '░' },
            BrushEntry { name: "City Marker".into(), preview_char: 'C' },
            BrushEntry { name: "Dungeon Marker".into(), preview_char: 'D' },
            BrushEntry { name: "Town Marker".into(), preview_char: 'T' },
        ],
        Some(MapType::Dungeon) => vec![
            BrushEntry { name: "Room".into(), preview_char: '#' },
            BrushEntry { name: "Corridor (H)".into(), preview_char: '.' },
            BrushEntry { name: "Corridor (V)".into(), preview_char: '.' },
            BrushEntry { name: "Door (closed)".into(), preview_char: '+' },
            BrushEntry { name: "Door (open)".into(), preview_char: '/' },
            BrushEntry { name: "Stairs (up)".into(), preview_char: '<' },
            BrushEntry { name: "Stairs (down)".into(), preview_char: '>' },
        ],
        Some(MapType::City) => vec![
            BrushEntry { name: "Building".into(), preview_char: '#' },
            BrushEntry { name: "Street".into(), preview_char: '\u{00B7}' },
            BrushEntry { name: "Plaza".into(), preview_char: '.' },
            BrushEntry { name: "Wall (H)".into(), preview_char: '\u{2500}' },
            BrushEntry { name: "Wall (V)".into(), preview_char: '\u{2502}' },
        ],
        Some(MapType::Building) => vec![
            BrushEntry { name: "Room".into(), preview_char: '#' },
            BrushEntry { name: "Furniture".into(), preview_char: '\u{2591}' },
            BrushEntry { name: "Door (closed)".into(), preview_char: '+' },
            BrushEntry { name: "Stairs (up)".into(), preview_char: '<' },
            BrushEntry { name: "Stairs (down)".into(), preview_char: '>' },
        ],
        None => vec![],
    }
}

fn activate_brush_by_index(app: &mut App, idx: usize) {
    let map_type = app.current_map().map(|m| m.map_type.clone());
    let brush: Box<dyn editor::brush::Brush> = match (map_type, idx) {
        (Some(MapType::Region), 0) => Box::new(editor::brushes::region::TerrainBrush::new('░', "terrain")),
        (Some(MapType::Region), 1) => Box::new(editor::brushes::region::CityMarkerBrush::new("New City", 60, 30)),
        (Some(MapType::Region), 2) => Box::new(editor::brushes::region::DungeonMarkerBrush::new("New Dungeon", 40, 20)),
        (Some(MapType::Region), 3) => Box::new(editor::brushes::region::TownMarkerBrush),

        (Some(MapType::Dungeon), 0) => Box::new(editor::brushes::dungeon::RoomBrush::new(5, 4)),
        (Some(MapType::Dungeon), 1) => Box::new(editor::brushes::dungeon::CorridorBrush::new(true, 6)),
        (Some(MapType::Dungeon), 2) => Box::new(editor::brushes::dungeon::CorridorBrush::new(false, 6)),
        (Some(MapType::Dungeon), 3) => Box::new(editor::brushes::dungeon::DoorBrush::new(false)),
        (Some(MapType::Dungeon), 4) => Box::new(editor::brushes::dungeon::DoorBrush::new(true)),
        (Some(MapType::Dungeon), 5) => Box::new(editor::brushes::dungeon::StairsBrush::new(true, app.current_layer + 1)),
        (Some(MapType::Dungeon), 6) => Box::new(editor::brushes::dungeon::StairsBrush::new(false, app.current_layer - 1)),

        (Some(MapType::City), 0) => Box::new(editor::brushes::city::BuildingBrush::new("Building", 8, 6)),
        (Some(MapType::City), 1) => Box::new(editor::brushes::city::StreetBrush),
        (Some(MapType::City), 2) => Box::new(editor::brushes::city::PlazaBrush),
        (Some(MapType::City), 3) => Box::new(editor::brushes::city::WallBrush::new(true)),
        (Some(MapType::City), 4) => Box::new(editor::brushes::city::WallBrush::new(false)),

        (Some(MapType::Building), 0) => Box::new(editor::brushes::dungeon::RoomBrush::new(5, 4)),
        (Some(MapType::Building), 1) => Box::new(editor::brushes::building::FurnitureBrush::new('\u{2591}', "Furniture")),
        (Some(MapType::Building), 2) => Box::new(editor::brushes::dungeon::DoorBrush::new(false)),
        (Some(MapType::Building), 3) => Box::new(editor::brushes::dungeon::StairsBrush::new(true, app.current_layer + 1)),
        (Some(MapType::Building), 4) => Box::new(editor::brushes::dungeon::StairsBrush::new(false, app.current_layer - 1)),

        _ => Box::new(editor::brush::NullBrush),
    };
    app.active_brush = brush;
}

// ──────────────────────────────────────────────────────────────────────────────
// Persistence helpers
// ──────────────────────────────────────────────────────────────────────────────

fn sync_stack_to_file(app: &App, map_file: &mut MapFile) {
    for m in &app.map_stack {
        map_file.upsert(m.clone());
    }
}

fn save_all(app: &App, map_file: &mut MapFile) {
    sync_stack_to_file(app, map_file);
    let path = app.maps_xml_path();
    if let Err(e) = map::xml::save(&path, map_file) {
        eprintln!("save error: {e}");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Layout helper
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use editor::brush::Brush;
    use map::{Cell, Layer, MapDef, MapType};
    use std::path::PathBuf;

    fn region_app() -> App {
        let mut m = MapDef::new("r1", "Region", MapType::Region, 20, 10);
        m.layers[0] = Layer::new(0, 0.0);
        App::new(PathBuf::from("/tmp"), Some(m))
    }

    // ── zone flood-fill ──────────────────────────────────────────────────────

    #[test]
    fn zone_flood_fill_assigns_zone_to_connected_cells() {
        let mut app = region_app();
        // Paint a 3×3 block of '.' cells
        {
            let map = app.current_map_mut().unwrap();
            let layer = map.layer_mut(0).unwrap();
            for r in 0u16..3 {
                for c in 0u16..3 {
                    layer.cells.insert((c, r), Cell::new('.').with_terrain("floor"));
                }
            }
            // An unconnected cell far away
            layer.cells.insert((10, 10), Cell::new('.').with_terrain("floor"));
        }
        app.cursor = (0, 0);
        zone_flood_fill(&mut app);

        let map = app.current_map().unwrap();
        let layer = map.layer(0).unwrap();
        // All 9 connected cells got a zone
        let zone = layer.cells[&(0, 0)].height_zone.clone().unwrap();
        for r in 0u16..3 {
            for c in 0u16..3 {
                assert_eq!(layer.cells[&(c, r)].height_zone.as_deref(), Some(zone.as_str()));
            }
        }
        // Unconnected cell: no zone
        assert!(layer.cells[&(10, 10)].height_zone.is_none());
        // Zone registered in layer
        assert!(layer.height_zones.contains_key(&zone));
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn zone_flood_fill_does_not_cross_different_char() {
        let mut app = region_app();
        {
            let map = app.current_map_mut().unwrap();
            let layer = map.layer_mut(0).unwrap();
            layer.cells.insert((0, 0), Cell::new('.'));
            layer.cells.insert((1, 0), Cell::new('#')); // barrier
            layer.cells.insert((2, 0), Cell::new('.'));
        }
        app.cursor = (0, 0);
        zone_flood_fill(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        let zone = layer.cells[&(0, 0)].height_zone.clone().unwrap();
        assert!(layer.cells[&(2, 0)].height_zone.as_deref() != Some(zone.as_str()));
    }

    // ── keying UUID storage ──────────────────────────────────────────────────

    #[test]
    fn keying_uuid_stored_in_cell() {
        // Tests the UUID-assignment logic directly without invoking emacsclient.
        let mut app = region_app();
        app.cursor = (3, 3);
        let z = app.current_layer;

        // Manually replicate what key_current_cell does (UUID assignment part only)
        let uuid = {
            let map = app.current_map_mut().unwrap();
            let layer = map.layer_mut(z).unwrap();
            let cell = layer.cells.entry((3, 3)).or_insert_with(|| Cell::new('.'));
            let uid = cell.key_uuid.get_or_insert_with(|| uuid::Uuid::new_v4().to_string());
            uid.clone()
        };

        let layer = app.current_map().unwrap().layer(z).unwrap();
        assert_eq!(layer.cells[&(3, 3)].key_uuid.as_deref(), Some(uuid.as_str()));
        assert!(!uuid.is_empty());
    }

    #[test]
    fn keying_city_building_stores_uuid_on_building() {
        let mut m = MapDef::new("city1", "City", MapType::City, 60, 30);
        m.layers[0] = Layer::new(0, 0.0);
        let mut b = map::Building::new("b1", "Tavern", 5, 3, 8, 6);
        b.key_uuid = Some("existing-uuid".to_string());
        m.buildings.push(b);
        let mut app = App::new(PathBuf::from("/tmp"), Some(m));
        // cursor inside building
        app.cursor = (6, 4);
        let z = app.current_layer;

        let uuid = {
            let map = app.current_map_mut().unwrap();
            if let Some(b) = map.buildings.iter_mut().find(|b| b.contains(6, 4)) {
                b.key_uuid.get_or_insert_with(|| uuid::Uuid::new_v4().to_string()).clone()
            } else { panic!("building not found") }
        };

        assert_eq!(uuid, "existing-uuid");
        // Cell should NOT get a key_uuid
        let layer = app.current_map().unwrap().layer(z).unwrap();
        assert!(layer.cells.get(&(6, 4)).and_then(|c| c.key_uuid.as_ref()).is_none());
    }

    // ── building interior canvas_clip constraint ─────────────────────────────

    #[test]
    fn furniture_brush_blocked_outside_canvas_clip() {
        use editor::brushes::building::FurnitureBrush;

        let mut m = MapDef::new("bld1", "Inn", MapType::Building, 20, 10);
        m.layers[0] = Layer::new(0, 0.0);
        let mut app = App::new(PathBuf::from("/tmp"), Some(m));
        app.canvas_clip = Some((2, 2, 8, 6)); // clip: cols 2-9, rows 2-7
        app.cursor = (0, 0); // outside clip
        let mut brush = FurnitureBrush::new('T', "Table");
        brush.on_confirm(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert!(layer.cells.get(&(0, 0)).is_none());
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn furniture_brush_allowed_inside_canvas_clip() {
        use editor::brushes::building::FurnitureBrush;

        let mut m = MapDef::new("bld2", "Inn", MapType::Building, 20, 10);
        m.layers[0] = Layer::new(0, 0.0);
        let mut app = App::new(PathBuf::from("/tmp"), Some(m));
        app.canvas_clip = Some((2, 2, 8, 6));
        app.cursor = (4, 4); // inside clip
        let mut brush = FurnitureBrush::new('T', "Table");
        brush.on_confirm(&mut app);

        let layer = app.current_map().unwrap().layer(0).unwrap();
        assert_eq!(layer.cells[&(4, 4)].ch, 'T');
    }

    // ── palette ──────────────────────────────────────────────────────────────

    #[test]
    fn palette_selected_char_returns_correct_char() {
        use ui::palette::{Palette, CATEGORIES};
        let mut p = Palette::default();
        // category 0 = Terrain, first char = '░'
        assert_eq!(p.selected_char(), Some(CATEGORIES[0].1[0]));
        p.move_right();
        assert_eq!(p.selected_char(), Some(CATEGORIES[0].1[1]));
    }

    #[test]
    fn palette_next_category_wraps() {
        use ui::palette::{Palette, CATEGORIES};
        let mut p = Palette::default();
        let n = CATEGORIES.len();
        for _ in 0..n {
            p.next_category();
        }
        assert_eq!(p.category, 0);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
