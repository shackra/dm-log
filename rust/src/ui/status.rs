use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
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
        if area.width < 10 {
            return;
        }

        let accent = self.app.accent_color();

        // Top 1px dim border
        for x in area.x..area.x + area.width {
            buf.set_string(
                x,
                area.y,
                "\u{2500}",
                Style::default().fg(Color::Indexed(236)),
            );
        }

        let y = if area.height > 1 { area.y + 1 } else { area.y };

        let mode_str = match self.app.mode {
            EditorMode::Normal => "DRAW",
            EditorMode::Brushing => "DRAW",
            EditorMode::Select => "SELECT",
            EditorMode::Key => "KEY",
            EditorMode::BrushPicker => "PICK",
            EditorMode::ZonePaint => "ZONE",
            EditorMode::ZoneEdit => "ZONE-EDIT",
        };

        let brush_name = self.app.active_brush.name();
        let brush_char = self.app.active_brush.preview_char();
        let (cx, cy) = self.app.cursor;
        let z = self.app.current_layer;

        // Compute absolute height
        let abs_h = if let Some(map) = self.app.current_map() {
            let cumbase = if z > 0 {
                map.cumulative_height(z - 1)
            } else {
                0.0
            };
            map.layer(z)
                .map(|l| l.cell_abs_height((cx, cy), cumbase))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Left side: accent-colored info
        // DRAW · [glyph] brush_name · col,row · Layer +z (+Xm)
        let left = if let Some(msg) = &self.app.status_msg {
            format!(
                " {mode_str} \u{00B7} [{brush_char}] {brush_name} \u{00B7} {cx},{cy} \u{00B7} Layer {:+} ({abs_h:+.0}m) \u{00B7} {msg}",
                z
            )
        } else {
            format!(
                " {mode_str} \u{00B7} [{brush_char}] {brush_name} \u{00B7} {cx},{cy} \u{00B7} Layer {:+} ({abs_h:+.0}m)",
                z
            )
        };

        buf.set_string(area.x, y, &left, Style::default().fg(accent));

        // Right side: dim keybinding hints
        let right =
            "hjkl move \u{00B7} b brush \u{00B7} PgUp/Dn layer \u{00B7} z zones \u{00B7} ? help ";
        let rx = (area.x + area.width).saturating_sub(right.len() as u16);
        if rx > area.x + left.len() as u16 {
            buf.set_string(rx, y, right, Style::default().fg(Color::Indexed(238)));
        }
    }
}
