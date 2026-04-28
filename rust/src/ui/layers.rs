use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::App;

/// Side-panel listing all layers sorted by z, showing height_m.
pub struct LayerPanel<'a> {
    pub app: &'a App,
}

impl<'a> LayerPanel<'a> {
    pub fn new(app: &'a App) -> Self {
        LayerPanel { app }
    }
}

impl<'a> Widget for LayerPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let title = "Layers";
        buf.set_string(area.x, area.y, title, Style::default().add_modifier(Modifier::BOLD));

        let map = match self.app.current_map() {
            Some(m) => m,
            None => return,
        };

        let mut layers: Vec<_> = map.layers.iter().collect();
        layers.sort_by_key(|l| l.z);

        for (i, layer) in layers.iter().enumerate() {
            let y = area.y + 1 + i as u16;
            if y >= area.y + area.height {
                break;
            }
            let active = layer.z == self.app.current_layer;
            let style = if active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
            };
            let label = format!(" z{:+2}  {:+.1}m ", layer.z, layer.height_m);
            buf.set_string(area.x, y, &label, style);
        }
    }
}
