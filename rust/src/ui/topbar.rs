use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::App;
use crate::map::MapType;

const MAP_TYPES: &[(&str, MapType)] = &[
    ("REGION", MapType::Region),
    ("DUNGEON", MapType::Dungeon),
    ("CITY", MapType::City),
    ("BUILDING", MapType::Building),
];

pub struct TopBar<'a> {
    pub app: &'a App,
}

impl<'a> TopBar<'a> {
    pub fn new(app: &'a App) -> Self {
        TopBar { app }
    }
}

impl<'a> Widget for TopBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 1 {
            return;
        }

        let accent = self.app.accent_color();

        // App name
        let title = " \u{2694} MAZAFORJA";
        buf.set_string(
            area.x,
            area.y,
            title,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        );

        // Separator
        let sep_x = area.x + title.len() as u16;
        buf.set_string(sep_x, area.y, " \u{2502} ", Style::default().fg(accent));

        let current_type = self.app.current_map().map(|m| &m.map_type);

        // Map type tabs
        let mut x = sep_x + 3;
        for (label, mt) in MAP_TYPES {
            let is_active = current_type == Some(mt);
            let style = if is_active {
                Style::default()
                    .fg(accent)
                    .bg(Color::Indexed(236))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Indexed(240))
            };
            let tab = format!(" {label} ");
            buf.set_string(x, area.y, &tab, style);
            x += tab.len() as u16 + 1;
            if x >= area.x + area.width {
                break;
            }
        }

        // Right side hints
        let right_text = "? help  q quit ";
        let rx = (area.x + area.width).saturating_sub(right_text.len() as u16);
        if rx > x {
            buf.set_string(
                rx,
                area.y,
                right_text,
                Style::default().fg(Color::Indexed(240)),
            );
        }
    }
}
