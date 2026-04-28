use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::App;

pub struct SidebarRight<'a> {
    pub app: &'a App,
}

impl<'a> SidebarRight<'a> {
    pub fn new(app: &'a App) -> Self {
        SidebarRight { app }
    }
}

impl<'a> Widget for SidebarRight<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 8 {
            return;
        }

        let accent = self.app.accent_color();

        // Left border
        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, "\u{2502}", Style::default().fg(accent));
        }

        let x = area.x + 2;
        let mut y = area.y;

        // MINIMAP header
        let header_style = Style::default()
            .fg(Color::Indexed(240))
            .add_modifier(Modifier::BOLD);
        buf.set_string(x, y, "MINIMAP", header_style);
        y += 1;

        // Minimap preview (simplified: show map dimensions + viewport indicator)
        if let Some(map) = self.app.current_map() {
            let preview_h = 6u16.min(area.height.saturating_sub(12));
            let preview_w = (area.width - 3).min(20);
            let scale_x = map.width as f32 / preview_w as f32;
            let scale_y = map.height as f32 / preview_h as f32;

            // Draw minimap cells (very simplified)
            let layer = map.layer(self.app.current_layer);
            for dy in 0..preview_h {
                for dx in 0..preview_w {
                    let map_x = (dx as f32 * scale_x) as u16;
                    let map_y = (dy as f32 * scale_y) as u16;
                    let has_cell = layer.and_then(|l| l.cells.get(&(map_x, map_y))).is_some();
                    let ch = if has_cell { '\u{2591}' } else { '\u{00B7}' };
                    let style = Style::default().fg(if has_cell {
                        accent
                    } else {
                        Color::Indexed(236)
                    });
                    let cell = buf.cell_mut((x + dx, y + dy));
                    if let Some(c) = cell {
                        c.set_char(ch);
                        c.set_style(style);
                    }
                }
            }

            // Viewport rectangle indicator
            let vp_x = (self.app.viewport.0 as f32 / scale_x).min((preview_w - 1) as f32) as u16;
            let vp_y = (self.app.viewport.1 as f32 / scale_y).min((preview_h - 1) as f32) as u16;
            let vp_cell = buf.cell_mut((x + vp_x, y + vp_y));
            if let Some(c) = vp_cell {
                c.set_style(
                    Style::default()
                        .fg(Color::Indexed(11))
                        .add_modifier(Modifier::BOLD),
                );
            }

            y += preview_h + 1;
        } else {
            y += 2;
        }

        // PROPERTIES section
        if y + 6 < area.y + area.height {
            buf.set_string(x, y, "PROPERTIES", header_style);
            y += 1;

            let dim = Style::default().fg(Color::Indexed(240));
            let val = Style::default().fg(Color::Indexed(250));

            if let Some(map) = self.app.current_map() {
                let props = [
                    ("Type", map.map_type.to_string()),
                    ("Name", map.name.clone()),
                    ("Size", format!("{}x{}", map.width, map.height)),
                    ("Layer", format!("z:{}", self.app.current_layer)),
                ];
                for (k, v) in &props {
                    if y >= area.y + area.height - 5 {
                        break;
                    }
                    buf.set_string(x, y, k, dim);
                    let vx = x + 7;
                    let max_w = (area.width - 3).saturating_sub(7) as usize;
                    let v_trunc: String = v.chars().take(max_w).collect();
                    buf.set_string(vx, y, &v_trunc, val);
                    y += 1;
                }
            }
            y += 1;
        }

        // Keyboard reference (bottom)
        let ref_lines = [
            "hjkl: move",
            "b: brush",
            "Tab: cycle",
            "z: zones",
            "PgUp/Dn: layer",
            "K: key cell",
            "?: help",
        ];
        let start_y = (area.y + area.height).saturating_sub(ref_lines.len() as u16 + 1);
        if start_y > y {
            let dim = Style::default().fg(Color::Indexed(238));
            for (i, line) in ref_lines.iter().enumerate() {
                buf.set_string(x, start_y + i as u16, line, dim);
            }
        }
    }
}
