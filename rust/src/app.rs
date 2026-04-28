use std::path::PathBuf;

use crate::editor::brush::Brush;
use crate::map::MapDef;

/// Current interaction mode of the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorMode {
    /// Default — cursor movement, layer switch, open menus.
    Normal,
    /// Actively drawing with the current brush.
    Brushing,
    /// Rectangular selection in progress.
    Select,
    /// Waiting for user to confirm keying (UUID assignment).
    Key,
    /// Brush-picker popup open.
    BrushPicker,
    /// Flood-fill zone-painting mode.
    ZonePaint,
    /// Editing an existing height zone (name / offset).
    ZoneEdit,
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Normal
    }
}

/// Top-level application state for the mazaforja TUI.
pub struct App {
    /// Navigation stack: index 0 = root map, last = current map.
    pub map_stack: Vec<MapDef>,
    /// Cursor position in (col, row) within the current map.
    pub cursor: (u16, u16),
    /// Top-left corner of the viewport (scroll offset).
    pub viewport: (u16, u16),
    /// The z-index of the layer currently being edited.
    pub current_layer: i32,
    /// Interaction mode.
    pub mode: EditorMode,
    /// Active brush (heap-allocated for trait-object dispatch).
    pub active_brush: Box<dyn Brush>,
    /// Campaign directory used for maps.xml and map.org paths.
    pub campaign_dir: PathBuf,
    /// Whether the CP437 character palette popup is visible.
    pub palette_open: bool,
    /// Whether the layer list side-panel is visible.
    pub layer_panel_open: bool,
    /// Optional rectangular clip for building-interior editing
    /// `(x, y, w, h)` in map coordinates.
    pub canvas_clip: Option<(u16, u16, u16, u16)>,
    /// Status message shown in the status bar (cleared each frame unless set).
    pub status_msg: Option<String>,
    /// Whether the editor should quit after the current frame.
    pub should_quit: bool,
}

impl App {
    pub fn new(campaign_dir: PathBuf, initial_map: Option<MapDef>) -> Self {
        let map_stack = initial_map.map(|m| vec![m]).unwrap_or_default();
        App {
            map_stack,
            cursor: (0, 0),
            viewport: (0, 0),
            current_layer: 0,
            mode: EditorMode::Normal,
            active_brush: Box::new(crate::editor::brush::NullBrush),
            campaign_dir,
            palette_open: false,
            layer_panel_open: false,
            canvas_clip: None,
            status_msg: None,
            should_quit: false,
        }
    }

    /// The map being edited right now (top of stack).
    pub fn current_map(&self) -> Option<&MapDef> {
        self.map_stack.last()
    }

    /// Mutable reference to the map being edited right now.
    pub fn current_map_mut(&mut self) -> Option<&mut MapDef> {
        self.map_stack.last_mut()
    }

    /// Push a child map onto the navigation stack and reset cursor/clip.
    pub fn push_map(&mut self, map: MapDef) {
        self.cursor = (0, 0);
        self.viewport = (0, 0);
        self.current_layer = 0;
        self.canvas_clip = None;
        self.map_stack.push(map);
    }

    /// Pop back to the parent map. Returns false if already at root.
    pub fn pop_map(&mut self) -> bool {
        if self.map_stack.len() <= 1 {
            return false;
        }
        self.map_stack.pop();
        self.cursor = (0, 0);
        self.viewport = (0, 0);
        self.current_layer = 0;
        self.canvas_clip = None;
        true
    }

    /// Move cursor by (dx, dy), clamped to current map bounds.
    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        if let Some(map) = self.current_map() {
            let max_x = map.width.saturating_sub(1);
            let max_y = map.height.saturating_sub(1);
            let new_x = (self.cursor.0 as i32 + dx).clamp(0, max_x as i32) as u16;
            let new_y = (self.cursor.1 as i32 + dy).clamp(0, max_y as i32) as u16;
            self.cursor = (new_x, new_y);
        }
    }

    /// Scroll viewport so cursor stays visible within the given terminal area.
    pub fn scroll_to_cursor(&mut self, term_w: u16, term_h: u16) {
        // Reserve space: 2 rows for status bar, 1 row for header.
        let view_w = term_w.saturating_sub(2);
        let view_h = term_h.saturating_sub(3);

        // Horizontal
        if self.cursor.0 < self.viewport.0 {
            self.viewport.0 = self.cursor.0;
        } else if self.cursor.0 >= self.viewport.0 + view_w {
            self.viewport.0 = self.cursor.0 - view_w + 1;
        }
        // Vertical
        if self.cursor.1 < self.viewport.1 {
            self.viewport.1 = self.cursor.1;
        } else if self.cursor.1 >= self.viewport.1 + view_h {
            self.viewport.1 = self.cursor.1 - view_h + 1;
        }
    }

    /// Switch to the previous layer (lower z).
    pub fn prev_layer(&mut self) {
        if let Some(map) = self.current_map() {
            let min_z = map.layers.iter().map(|l| l.z).min().unwrap_or(0);
            if self.current_layer > min_z {
                self.current_layer -= 1;
            }
        }
    }

    /// Switch to the next layer (higher z).
    pub fn next_layer(&mut self) {
        if let Some(map) = self.current_map() {
            let max_z = map.layers.iter().map(|l| l.z).max().unwrap_or(0);
            if self.current_layer < max_z {
                self.current_layer += 1;
            }
        }
    }

    /// Set a transient status message.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    /// Path to maps.xml for the current campaign.
    pub fn maps_xml_path(&self) -> PathBuf {
        self.campaign_dir.join("maps.xml")
    }

    /// Path to map.org for the current campaign.
    pub fn map_org_path(&self) -> PathBuf {
        self.campaign_dir.join("map.org")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Layer, MapDef, MapType};

    fn make_app(w: u16, h: u16) -> App {
        let map = MapDef::new("test", "Test", MapType::Region, w, h);
        App::new(PathBuf::from("/tmp"), Some(map))
    }

    // ── cursor ──────────────────────────────────────────────────────────────

    #[test]
    fn cursor_clamps_at_map_edge() {
        let mut app = make_app(10, 8);
        app.move_cursor(-100, -100);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(100, 100);
        assert_eq!(app.cursor, (9, 7));
    }

    #[test]
    fn cursor_moves_incrementally() {
        let mut app = make_app(10, 10);
        app.move_cursor(3, 2);
        assert_eq!(app.cursor, (3, 2));
        app.move_cursor(-1, 1);
        assert_eq!(app.cursor, (2, 3));
    }

    // ── viewport scroll ─────────────────────────────────────────────────────

    #[test]
    fn scroll_to_cursor_keeps_cursor_in_view() {
        let mut app = make_app(80, 40);
        // Move cursor far right/down
        app.cursor = (70, 35);
        app.scroll_to_cursor(20, 10);
        // cursor must be within viewport + view area
        let view_w = 20u16.saturating_sub(2);
        let view_h = 10u16.saturating_sub(3);
        assert!(app.cursor.0 >= app.viewport.0);
        assert!(app.cursor.0 < app.viewport.0 + view_w);
        assert!(app.cursor.1 >= app.viewport.1);
        assert!(app.cursor.1 < app.viewport.1 + view_h);
    }

    // ── layer navigation ────────────────────────────────────────────────────

    #[test]
    fn layer_nav_stays_within_existing_layers() {
        let mut app = make_app(10, 10);
        // only z=0 exists — prev/next should be no-ops
        app.prev_layer();
        assert_eq!(app.current_layer, 0);
        app.next_layer();
        assert_eq!(app.current_layer, 0);

        // add z=1
        app.current_map_mut().unwrap().layers.push(Layer::new(1, 3.0));
        app.next_layer();
        assert_eq!(app.current_layer, 1);
        app.next_layer(); // no z=2 → should stay
        assert_eq!(app.current_layer, 1);
        app.prev_layer();
        assert_eq!(app.current_layer, 0);
    }

    // ── map stack ───────────────────────────────────────────────────────────

    #[test]
    fn push_pop_map_stack() {
        let mut app = make_app(80, 40);
        assert_eq!(app.map_stack.len(), 1);

        let child = MapDef::new("child", "Child", MapType::Dungeon, 20, 10);
        app.push_map(child);
        assert_eq!(app.map_stack.len(), 2);
        assert_eq!(app.current_map().unwrap().id, "child");
        assert_eq!(app.cursor, (0, 0));

        let popped = app.pop_map();
        assert!(popped);
        assert_eq!(app.map_stack.len(), 1);

        // pop at root returns false
        assert!(!app.pop_map());
        assert_eq!(app.map_stack.len(), 1);
    }

    #[test]
    fn push_map_resets_cursor_and_clip() {
        let mut app = make_app(80, 40);
        app.cursor = (5, 5);
        app.canvas_clip = Some((1, 1, 10, 10));

        let child = MapDef::new("c2", "C2", MapType::Building, 20, 10);
        app.push_map(child);
        assert_eq!(app.cursor, (0, 0));
        assert!(app.canvas_clip.is_none());
    }

    // ── status message ──────────────────────────────────────────────────────

    #[test]
    fn set_status_stores_message() {
        let mut app = make_app(10, 10);
        app.set_status("hello");
        assert_eq!(app.status_msg.as_deref(), Some("hello"));
    }

    // ── paths ────────────────────────────────────────────────────────────────

    #[test]
    fn campaign_paths_correct() {
        let app = App::new(PathBuf::from("/campaigns/test"), None);
        assert_eq!(app.maps_xml_path(), PathBuf::from("/campaigns/test/maps.xml"));
        assert_eq!(app.map_org_path(), PathBuf::from("/campaigns/test/map.org"));
    }
}
