// Copyright (C) 2026 Raimo Geisel
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Process list tab renderer.
//!
//! Renders the top-100 CPU-sorted process list as a table with vertical
//! scrolling. [`draw_table`] is also called by the Overview tab.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::app::App;
use crate::ui::helpers::scroll_indicator;

/// Renders the Processes tab in `area`, computing the visible row count and
/// scroll indicator from the current [`App::process_scroll`] offset.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible = (area.height as usize).saturating_sub(3);
    let total = app.processes.len();
    let scroll = app.process_scroll;

    let can_up = scroll > 0;
    let can_down = scroll + visible < total;
    let indicator = scroll_indicator(can_up, can_down);
    let title = format!(" {}{indicator} ", app.t.processes_title);

    draw_table(frame, app, area, visible, &title);
}

/// Renders up to `limit` process rows starting at [`App::process_scroll`]
/// inside a bordered table titled `title`.
///
/// Called by both the Processes tab and the Overview tab.
pub fn draw_table(frame: &mut Frame, app: &App, area: Rect, limit: usize, title: &str) {
    let header = Row::new(vec![
        Cell::from(app.t.col_pid.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(app.t.col_name.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(app.t.col_cpu_pct.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(app.t.col_mem_kib.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
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
