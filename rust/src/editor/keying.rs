use std::process::Command;

use crate::app::App;
use crate::map::MapType;

/// Assign a UUID to the cell (or building) at the cursor.
/// Suspends ratatui, opens Emacs with the keying buffer, then reinits.
pub fn key_current_cell(app: &mut App) {
    let (col, row) = app.cursor;
    let z = app.current_layer;

    // Determine what type of entry we're keying
    let entity_type = match app.current_map().map(|m| &m.map_type) {
        Some(MapType::Region) => "hex",
        Some(MapType::Dungeon) => "room",
        Some(MapType::City) => "zone",
        Some(MapType::Building) => "room",
        None => {
            app.set_status("No map loaded.");
            return;
        }
    };

    // Assign or reuse UUID
    let uuid = {
        let map = match app.current_map_mut() {
            Some(m) => m,
            None => return,
        };

        // City map: prefer building UUID if cursor is inside a building
        if map.map_type == MapType::City {
            if let Some(b) = map.buildings.iter_mut().find(|b| b.contains(col, row)) {
                let uid = b
                    .key_uuid
                    .get_or_insert_with(|| uuid::Uuid::new_v4().to_string());
                uid.clone()
            } else if let Some(layer) = map.layer_mut(z) {
                let cell = layer
                    .cells
                    .entry((col, row))
                    .or_insert_with(|| crate::map::Cell::new('.'));
                let uid = cell
                    .key_uuid
                    .get_or_insert_with(|| uuid::Uuid::new_v4().to_string());
                uid.clone()
            } else {
                return;
            }
        } else if let Some(layer) = map.layer_mut(z) {
            let cell = layer
                .cells
                .entry((col, row))
                .or_insert_with(|| crate::map::Cell::new('.'));
            let uid = cell
                .key_uuid
                .get_or_insert_with(|| uuid::Uuid::new_v4().to_string());
            uid.clone()
        } else {
            return;
        }
    };

    let campaign_dir = app.campaign_dir.to_string_lossy().into_owned();

    // Save XML before suspending
    let maps_path = app.maps_xml_path();
    let map_file = collect_map_file(app);
    let _ = crate::map::xml::save(&maps_path, &map_file);

    // Suspend ratatui, open Emacs
    let _ = ratatui::restore();
    open_keying_buffer(&uuid, entity_type, &campaign_dir);

    // Reinit terminal
    let _ = ratatui::init();

    app.set_status(format!("Keyed: {uuid}"));
}

/// Collect all maps in the stack into a temporary MapFile for saving.
fn collect_map_file(app: &App) -> crate::map::MapFile {
    let mut mf = crate::map::MapFile::new();
    for m in &app.map_stack {
        mf.upsert(m.clone());
    }
    mf
}

fn open_keying_buffer(uuid: &str, entity_type: &str, campaign_dir: &str) {
    let elisp =
        format!("(dm-log-map--open-keying-buffer \"{uuid}\" \"{entity_type}\" \"{campaign_dir}\")");

    // Try emacsclient first, fall back to emacs
    let ok = Command::new("emacsclient")
        .args(["--eval", &elisp])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        let _ = Command::new("emacs").args(["--eval", &elisp]).status();
    }
}
