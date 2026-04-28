use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

/// Categories of CP437-compatible characters available in the palette.
pub struct Palette {
    pub selected_row: usize,
    pub selected_col: usize,
    pub category: usize,
}

pub const CATEGORIES: &[(&str, &[char])] = &[
    (
        "Terrain",
        &['░', '▒', '▓', '█', '~', '≈', '▲', '·', ',', '"'],
    ),
    (
        "Dungeon",
        &['#', '.', '+', '/', '\\', '|', '-', '@', '^', '<', '>'],
    ),
    (
        "Box Drawing",
        &[
            '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '═', '║', '╔', '╗', '╚', '╝',
            '╠', '╣', '╦', '╩', '╬',
        ],
    ),
    ("Markers", &['C', 'D', 'T', 'B', 'M']),
    ("Flora", &['♠', '♣', '♦', '♥', '☼', '☺', '☻', '•', '◘', '○']),
    ("Arrows", &['←', '→', '↑', '↓', '↔', '↕']),
];

const COLS: usize = 10;

impl Default for Palette {
    fn default() -> Self {
        Palette {
            selected_row: 0,
            selected_col: 0,
            category: 0,
        }
    }
}

impl Palette {
    pub fn selected_char(&self) -> Option<char> {
        let chars = CATEGORIES.get(self.category)?.1;
        let idx = self.selected_row * COLS + self.selected_col;
        chars.get(idx).copied()
    }

    pub fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }
    pub fn move_down(&mut self) {
        let max_row = (CATEGORIES[self.category].1.len() + COLS - 1) / COLS;
        if self.selected_row + 1 < max_row {
            self.selected_row += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
        }
    }
    pub fn move_right(&mut self) {
        let chars_in_row = {
            let chars = CATEGORIES[self.category].1;
            let start = self.selected_row * COLS;
            chars.len().saturating_sub(start).min(COLS)
        };
        if self.selected_col + 1 < chars_in_row {
            self.selected_col += 1;
        }
    }
    pub fn next_category(&mut self) {
        self.category = (self.category + 1) % CATEGORIES.len();
        self.selected_row = 0;
        self.selected_col = 0;
    }
    pub fn prev_category(&mut self) {
        self.category = (self.category + CATEGORIES.len() - 1) % CATEGORIES.len();
        self.selected_row = 0;
        self.selected_col = 0;
    }
}

/// Ratatui widget for the palette popup.
pub struct PaletteWidget<'a> {
    pub palette: &'a Palette,
}

impl<'a> Widget for PaletteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Palette (Tab=category, hjkl=move, Enter=select) ")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        // Category tabs
        let (cat_name, chars) = CATEGORIES[self.palette.category];
        let tab_line = format!("[{}] {cat_name}", self.palette.category + 1);
        buf.set_string(
            inner.x,
            inner.y,
            &tab_line,
            Style::default().add_modifier(Modifier::BOLD),
        );

        let row_start = inner.y + 1;
        for (i, chunk) in chars.chunks(COLS).enumerate() {
            let y = row_start + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            for (j, &ch) in chunk.iter().enumerate() {
                let x = inner.x + j as u16 * 2;
                if x + 2 > inner.x + inner.width {
                    break;
                }
                let selected = i == self.palette.selected_row && j == self.palette.selected_col;
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let cell_buf = buf.cell_mut((x, y));
                if let Some(c) = cell_buf {
                    c.set_char(ch);
                    c.set_style(style);
                }
            }
        }
    }
}
