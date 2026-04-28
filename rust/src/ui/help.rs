use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

const BINDINGS: &[(&str, &str)] = &[
    ("Move", "hjkl / arrows"),
    ("Brush picker", "b"),
    ("Palette", "p"),
    ("Layer panel", "L"),
    ("Prev / next layer", "[ / ]"),
    ("Add layer", "+ / -"),
    ("Zone paint", "Z"),
    ("Key cell", "K"),
    ("Drill down", "Enter"),
    ("Drill up / back", "Esc"),
    ("Quit + save", "q"),
];

pub struct HelpWidget;

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Help  (any key to close) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        let inner = block.inner(area);
        block.render(area, buf);

        let col_w: u16 = 20;
        for (i, (action, key)) in BINDINGS.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            buf.set_string(
                inner.x + 1,
                y,
                action,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            buf.set_string(
                inner.x + 1 + col_w,
                y,
                key,
                Style::default().fg(Color::White),
            );
        }
    }
}
