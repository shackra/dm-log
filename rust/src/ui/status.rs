use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::{App, EditorMode};

pub struct StatusBar<'a> {
    pub app: &'a App,
}

impl<'a> StatusBar<'a> {
    pub fn new(app: &'a App) -> Self {
        StatusBar { app }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mode_str = match self.app.mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Brushing => "BRUSH",
            EditorMode::Select => "SELECT",
            EditorMode::Key => "KEY",
            EditorMode::BrushPicker => "PICK",
            EditorMode::ZonePaint => "ZONE",
            EditorMode::ZoneEdit => "ZONE-EDIT",
        };

        let brush_name = self.app.active_brush.name();
        let (cx, cy) = self.app.cursor;
        let z = self.app.current_layer;

        // Compute absolute height of cursor cell
        let abs_h = if let Some(map) = self.app.current_map() {
            let cumbase = if z > 0 {
                map.cumulative_height(z - 1)
            } else {
                0.0
            };
            if let Some(layer) = map.layer(z) {
                layer.cell_abs_height((cx, cy), cumbase)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Zone name at cursor
        let zone_str = if let Some(map) = self.app.current_map() {
            if let Some(layer) = map.layer(z) {
                if let Some(cell) = layer.cells.get(&(cx, cy)) {
                    cell.height_zone.as_deref().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let map_name = self.app.current_map().map(|m| m.name.as_str()).unwrap_or("—");

        let main = if zone_str.is_empty() {
            format!(
                " [{mode_str}/{brush_name}] {cx},{cy} z:{z} ({abs_h:.1}m)  {map_name}"
            )
        } else {
            format!(
                " [{mode_str}/{brush_name}] {cx},{cy} z:{z} ({abs_h:.1}m) zone:{zone_str}  {map_name}"
            )
        };

        let style = Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD);
        buf.set_string(area.x, area.y, &main, style);

        // Transient status message on second row if available
        if let Some(msg) = &self.app.status_msg {
            if area.height > 1 {
                buf.set_string(area.x, area.y + 1, format!(" {msg}"), Style::default().fg(Color::Yellow));
            }
        }
    }
}
