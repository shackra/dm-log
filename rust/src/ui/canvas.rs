use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::{App, EditorMode};

/// The main map grid widget.
pub struct Canvas<'a> {
    pub app: &'a App,
}

impl<'a> Canvas<'a> {
    pub fn new(app: &'a App) -> Self {
        Canvas { app }
    }
}

impl<'a> Widget for Canvas<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let map = match self.app.current_map() {
            Some(m) => m,
            None => {
                buf.set_string(area.x, area.y, "No map loaded. Press 'n' to create one.", Style::default());
                return;
            }
        };

        let layer = map.layer(self.app.current_layer);
        let (vx, vy) = self.app.viewport;
        let cursor = self.app.cursor;

        for row in 0..area.height {
            for col in 0..area.width {
                let map_col = vx + col;
                let map_row = vy + row;

                // Out of map bounds → dim background
                if map_col >= map.width || map_row >= map.height {
                    let cell_buf = buf.cell_mut((area.x + col, area.y + row));
                    if let Some(c) = cell_buf {
                        c.set_char(' ');
                        c.set_style(Style::default().bg(Color::DarkGray));
                    }
                    continue;
                }

                let screen_x = area.x + col;
                let screen_y = area.y + row;

                // Get cell from current layer
                let (ch, style) = if let Some(layer) = layer {
                    if let Some(cell) = layer.cells.get(&(map_col, map_row)) {
                        let mut s = Style::default();
                        if cell.locked {
                            s = s.fg(Color::Red).add_modifier(Modifier::DIM);
                        } else if cell.key_uuid.is_some() {
                            s = s.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                        }
                        // Height zone tint
                        if cell.height_zone.is_some() {
                            s = s.fg(Color::Cyan);
                        }
                        (cell.ch, s)
                    } else {
                        (' ', Style::default())
                    }
                } else {
                    (' ', Style::default())
                };

                // Building overlay for City maps
                let style = if crate::map::MapType::City == map.map_type {
                    if map.building_at(map_col, map_row).is_some() {
                        // Building area gets a subtle tint if not already styled
                        if ch == ' ' {
                            let cell_buf = buf.cell_mut((screen_x, screen_y));
                            if let Some(c) = cell_buf {
                                c.set_char('·');
                                c.set_style(Style::default().fg(Color::DarkGray));
                            }
                            continue;
                        }
                        style
                    } else {
                        style
                    }
                } else {
                    style
                };

                // Cursor: reverse video
                let style = if (map_col, map_row) == cursor {
                    style.add_modifier(Modifier::REVERSED)
                } else {
                    style
                };

                // Brush preview: dim if brushing and near cursor
                let style = if self.app.mode == EditorMode::Brushing
                    && (map_col, map_row) == cursor
                {
                    style.add_modifier(Modifier::DIM)
                } else {
                    style
                };

                let cell_buf = buf.cell_mut((screen_x, screen_y));
                if let Some(c) = cell_buf {
                    c.set_char(if ch == '\0' { ' ' } else { ch });
                    c.set_style(style);
                }
            }
        }
    }
}
