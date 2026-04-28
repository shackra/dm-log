use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::App;

pub struct LayerBar<'a> {
    pub app: &'a App,
}

impl<'a> LayerBar<'a> {
    pub fn new(app: &'a App) -> Self {
        LayerBar { app }
    }
}

impl<'a> Widget for LayerBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 2 {
            return;
        }

        let accent = self.app.accent_color();

        // Top border
        for x in area.x..area.x + area.width {
            buf.set_string(x, area.y, "\u{2500}", Style::default().fg(accent));
        }

        let y = area.y + 1;

        // Left side: +/- buttons
        let dim = Style::default().fg(Color::Indexed(240));
        buf.set_string(area.x + 1, y, "+ add", dim);
        if area.height > 2 {
            buf.set_string(area.x + 1, y + 1, "- rem", dim);
        }

        let map = match self.app.current_map() {
            Some(m) => m,
            None => return,
        };

        // Layer slots — sorted by z descending (highest first)
        let mut layers: Vec<_> = map.layers.iter().collect();
        layers.sort_by(|a, b| b.z.cmp(&a.z));

        let slot_w = 12u16;
        let start_x = area.x + 8;
        let mut sx = start_x;

        for layer in &layers {
            if sx + slot_w > area.x + area.width {
                break;
            }

            let is_active = layer.z == self.app.current_layer;
            let is_ground = layer.z == 0;

            let (num_style, name_style, bg) = if is_active {
                (
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                    Color::Indexed(236),
                )
            } else if is_ground {
                (
                    Style::default().fg(Color::Indexed(3)), // brown/amber
                    Style::default().fg(Color::Indexed(3)),
                    Color::Reset,
                )
            } else {
                (
                    Style::default().fg(Color::Indexed(240)),
                    Style::default().fg(Color::Indexed(245)),
                    Color::Reset,
                )
            };

            // Active top border
            if is_active {
                for x in sx..sx + slot_w {
                    buf.set_string(x, area.y, "\u{2501}", Style::default().fg(accent));
                }
            }

            // Fill background
            if bg != Color::Reset {
                for x in sx..sx + slot_w {
                    for dy in 0..area.height.saturating_sub(1) {
                        if let Some(c) = buf.cell_mut((x, y + dy)) {
                            c.set_bg(bg);
                        }
                    }
                }
            }

            // Layer number (top-left)
            let num_str = format!("{:+}", layer.z);
            buf.set_string(sx + 1, y, &num_str, num_style);

            // Height (top-right)
            let h_str = format!("{:+.0}m", layer.height_m);
            let hx = (sx + slot_w).saturating_sub(h_str.len() as u16 + 1);
            buf.set_string(hx, y, &h_str, Style::default().fg(Color::Indexed(238)));

            // Layer name
            if area.height > 2 {
                let name = if is_ground {
                    "Ground"
                } else if layer.z > 0 {
                    "Above"
                } else {
                    "Below"
                };
                buf.set_string(sx + 1, y + 1, name, name_style);
            }

            sx += slot_w + 1;
        }

        // Right side: z: zones button
        let zx = (area.x + area.width).saturating_sub(10);
        if zx > sx {
            let zone_style = if self.app.mode == crate::app::EditorMode::ZonePaint {
                Style::default().fg(accent).add_modifier(Modifier::BOLD)
            } else {
                dim
            };
            buf.set_string(zx, y, "z: zones", zone_style);
        }
    }
}
