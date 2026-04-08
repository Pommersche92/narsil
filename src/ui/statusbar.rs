use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let inv = Style::default().add_modifier(Modifier::REVERSED);
    let inv_bold = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);

    let mut bindings: Vec<(&str, &str)> = vec![
        ("Tab / → / l", "Navigate tabs right"),
        ("Shift+Tab / ← / h", "Navigate tabs left"),
        ("1-7", "Jump to tab"),
        ("q / Ctrl-C", "Quit"),
    ];

    if matches!(app.selected_tab, 4 | 5 | 6) {
        bindings.extend_from_slice(&[("↑ / k", "Scroll up"), ("↓ / j", "Scroll down")]);
    }

    let mut spans: Vec<Span> = Vec::new();
    for (i, (keys, action)) in bindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  |  ", inv));
        }
        spans.push(Span::styled(format!(" {keys}"), inv_bold));
        spans.push(Span::styled(format!(": {action} "), inv));
    }

    let bar = Paragraph::new(Line::from(spans)).style(inv);
    frame.render_widget(bar, area);
}
