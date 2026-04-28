use crate::app::App;
use crate::editor::brush::Brush;
use crate::map::Cell;

/// Paints a single furniture character inside a Building map.
pub struct FurnitureBrush {
    pub ch: char,
    pub label: String,
}

impl FurnitureBrush {
    pub fn new(ch: char, label: impl Into<String>) -> Self {
        FurnitureBrush {
            ch,
            label: label.into(),
        }
    }
}

impl Brush for FurnitureBrush {
    fn name(&self) -> &str {
        &self.label
    }
    fn preview_char(&self) -> char {
        self.ch
    }
    fn on_move(&mut self, _app: &mut App) {}

    fn on_confirm(&mut self, app: &mut App) {
        let (col, row) = app.cursor;
        let z = app.current_layer;

        // Reject if outside canvas_clip
        if let Some((cx, cy, cw, ch)) = app.canvas_clip {
            if col < cx || col >= cx + cw || row < cy || row >= cy + ch {
                app.set_status("Outside building bounds.");
                return;
            }
        }

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
                        .insert((col, row), Cell::new(self.ch).with_terrain("furniture"));
                }
            }
        }
    }

    fn cancel(&mut self, _app: &mut App) {}
}
