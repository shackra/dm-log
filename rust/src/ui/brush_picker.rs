use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

/// An entry in the brush picker list.
pub struct BrushEntry {
    pub name: String,
    pub preview_char: char,
}

/// Brush-picker popup widget.
pub struct BrushPickerWidget<'a> {
    pub entries: &'a [BrushEntry],
    pub selected: usize,
}

impl<'a> Widget for BrushPickerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Brushes (hjkl=move, Enter=select, Esc=cancel) ")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        for (i, entry) in self.entries.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let selected = i == self.selected;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let line = format!(" {} {}  ", entry.preview_char, entry.name);
            buf.set_string(inner.x, y, &line, style);
        }
    }
}
