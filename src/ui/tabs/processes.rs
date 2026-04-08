use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::app::App;
use crate::ui::helpers::scroll_indicator;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible = (area.height as usize).saturating_sub(3);
    let total = app.processes.len();
    let scroll = app.process_scroll;

    let can_up = scroll > 0;
    let can_down = scroll + visible < total;
    let indicator = scroll_indicator(can_up, can_down);
    let title = format!(" Processes (sorted by CPU){indicator} ");

    draw_table(frame, app, area, visible, &title);
}

pub fn draw_table(frame: &mut Frame, app: &App, area: Rect, limit: usize, title: &str) {
    let header = Row::new(vec![
        Cell::from("PID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("CPU %").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Mem (KiB)").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::Yellow));

    let scroll = app.process_scroll;
    let rows: Vec<Row> = app
        .processes
        .iter()
        .skip(scroll)
        .take(limit)
        .map(|p| {
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}", p.cpu)),
                Cell::from(format!("{}", p.mem_kb)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().title(title).borders(Borders::ALL))
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, area);
}
