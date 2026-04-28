use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::app::App;
use crate::map::MapType;

/// CP437 brush definitions per map type: (glyph, label, palette color index)
pub const REGION_BRUSHES: &[(char, &str, u8)] = &[
    ('\u{00B7}', "Plains", 7),
    ('\u{2663}', "Forest", 2),
    ('\u{25B2}', "Mountain", 7),
    ('\u{2591}', "Hills", 7),
    ('\u{2248}', "Water", 6),
    ('\u{2261}', "Road", 3),
    ('\u{25CB}', "City", 11),
    ('\u{2022}', "Village", 7),
    ('\u{263B}', "Danger", 9),
    ('\u{2020}', "Temple", 14),
    ('\u{2592}', "Swamp", 2),
    ('\u{2588}', "Impassable", 8),
];

pub const DUNGEON_BRUSHES: &[(char, &str, u8)] = &[
    ('\u{2588}', "Wall", 8),
    ('\u{2591}', "Rough Wall", 8),
    ('\u{00B7}', "Floor", 7),
    ('+', "Door", 3),
    ('\u{2550}', "Locked Door", 3),
    ('\u{25BA}', "Stair Down", 7),
    ('\u{25C4}', "Stair Up", 7),
    ('\u{25AA}', "Pillar", 7),
    ('\u{00D7}', "Trap", 9),
    ('$', "Chest", 11),
    ('\u{263B}', "Monster", 9),
    ('\u{263C}', "Torch", 11),
];

pub const CITY_BRUSHES: &[(char, &str, u8)] = &[
    ('\u{00B7}', "Street", 7),
    ('\u{2550}', "Road", 3),
    ('\u{2551}', "Wall V", 7),
    ('\u{2554}', "Corner", 7),
    ('+', "Gate", 3),
    ('\u{25AA}', "Market", 10),
    ('\u{2020}', "Temple", 14),
    ('\u{263C}', "Inn/Tavern", 10),
    ('\u{25CB}', "Well", 6),
    ('\u{2591}', "Park", 2),
    ('\u{2593}', "Wall thick", 8),
    ('\u{2261}', "Bridge", 3),
];

pub const BUILDING_BRUSHES: &[(char, &str, u8)] = &[
    ('\u{2551}', "Wall V", 6),
    ('\u{2550}', "Wall H", 6),
    ('\u{2554}', "Corner TL", 6),
    ('\u{00B7}', "Floor", 7),
    ('+', "Door", 3),
    ('\u{25AA}', "Furniture", 3),
    ('\u{256A}', "Table", 3),
    ('\u{25CB}', "Window", 14),
    ('\u{2261}', "Stairs", 7),
    ('\u{263C}', "Light", 11),
    ('\u{2591}', "Rug/Mat", 3),
];

pub fn brushes_for(map_type: &MapType) -> &'static [(char, &'static str, u8)] {
    match map_type {
        MapType::Region => REGION_BRUSHES,
        MapType::Dungeon => DUNGEON_BRUSHES,
        MapType::City => CITY_BRUSHES,
        MapType::Building => BUILDING_BRUSHES,
    }
}

/// Color palette indices per map type
pub fn palette_for(map_type: &MapType) -> &'static [u8] {
    match map_type {
        MapType::Region => &[0, 2, 3, 6, 7, 8, 9, 10, 11, 14],
        MapType::Dungeon => &[0, 3, 7, 8, 9, 11],
        MapType::City => &[0, 2, 3, 6, 7, 8, 10, 11, 14],
        MapType::Building => &[0, 3, 6, 7, 8, 11, 14],
    }
}

pub struct SidebarLeft<'a> {
    pub app: &'a App,
    pub selected_brush: usize,
    pub selected_color: usize,
}

impl<'a> SidebarLeft<'a> {
    pub fn new(app: &'a App, selected_brush: usize, selected_color: usize) -> Self {
        SidebarLeft {
            app,
            selected_brush,
            selected_color,
        }
    }
}

impl<'a> Widget for SidebarLeft<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 5 {
            return;
        }

        let accent = self.app.accent_color();
        let map_type = match self.app.current_map().map(|m| &m.map_type) {
            Some(t) => t.clone(),
            None => return,
        };

        // Right border
        let bx = area.x + area.width - 1;
        for y in area.y..area.y + area.height {
            buf.set_string(bx, y, "\u{2502}", Style::default().fg(accent));
        }

        let inner_w = area.width.saturating_sub(2);
        let mut y = area.y;

        // Header: BRUSHES · CP437
        buf.set_string(
            area.x + 1,
            y,
            "BRUSHES \u{00B7} CP437",
            Style::default()
                .fg(Color::Indexed(240))
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Brush list
        let brushes = brushes_for(&map_type);
        for (i, (glyph, label, color_idx)) in brushes.iter().enumerate() {
            if y >= area.y + area.height - 5 {
                break;
            }
            let is_selected = i == self.selected_brush;
            let glyph_style = Style::default().fg(Color::Indexed(*color_idx));
            let label_style = if is_selected {
                Style::default().fg(accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Indexed(240))
            };

            // Selected indicator
            if is_selected {
                buf.set_string(area.x, y, "\u{2502}", Style::default().fg(accent));
            }

            buf.set_string(area.x + 1, y, &format!(" {}", glyph), glyph_style);
            let label_text: String = label
                .chars()
                .take((inner_w as usize).saturating_sub(4))
                .collect();
            buf.set_string(area.x + 4, y, &label_text, label_style);
            y += 1;
        }

        y += 1;
        if y >= area.y + area.height - 2 {
            return;
        }

        // Color palette
        buf.set_string(
            area.x + 1,
            y,
            "PALETTE",
            Style::default()
                .fg(Color::Indexed(240))
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        let palette = palette_for(&map_type);
        let mut px = area.x + 1;
        for (i, &idx) in palette.iter().enumerate() {
            if px + 2 > area.x + area.width - 1 {
                break;
            }
            let is_selected = i == self.selected_color;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Indexed(idx))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::Indexed(idx))
                    .fg(Color::Indexed(idx))
            };
            buf.set_string(px, y, "\u{2588}\u{2588}", style);
            px += 3;
        }
    }
}
