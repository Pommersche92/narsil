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

//! Memory tab renderer.
//!
//! Shows a 60-second RAM usage chart on the top half, and RAM/swap
//! [`SplitGauge`] bars on the bottom half.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};

use crate::app::App;
use crate::metrics::HISTORY_LEN;
use crate::ui::helpers::usage_color_f64;
use crate::ui::widgets::SplitGauge;

/// Renders the full Memory tab (history chart + RAM/swap gauges) in `area`.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let data: Vec<(f64, f64)> = app
        .mem
        .history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let datasets = vec![Dataset::default()
        .name("Mem %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    " Memory Usage History ",
                    Style::default().fg(Color::Green),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, HISTORY_LEN as f64])
                .labels(vec![
                    Span::raw("60s ago"),
                    Span::raw("30s ago"),
                    Span::raw("now"),
                ]),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec![Span::raw("0%"), Span::raw("50%"), Span::raw("100%")]),
        );

    frame.render_widget(chart, chunks[0]);

    let used_gb = app.mem.used as f64 / 1_073_741_824.0;
    let total_gb = app.mem.total as f64 / 1_073_741_824.0;
    let swap_used_gb = app.mem.swap_used as f64 / 1_073_741_824.0;
    let swap_total_gb = app.mem.swap_total as f64 / 1_073_741_824.0;
    let mem_pct = if app.mem.total > 0 {
        app.mem.used as f64 / app.mem.total as f64
    } else {
        0.0
    };
    let swap_pct = if app.mem.swap_total > 0 {
        app.mem.swap_used as f64 / app.mem.swap_total as f64
    } else {
        0.0
    };

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    let mem_gauge = SplitGauge::new(
        mem_pct,
        usage_color_f64(mem_pct * 100.0),
        format!("{:.0}%", mem_pct * 100.0),
    )
    .block(
        Block::default()
            .title(format!(" RAM  {used_gb:.1} / {total_gb:.1} GiB "))
            .borders(Borders::ALL),
    );

    let swap_gauge = SplitGauge::new(
        swap_pct,
        Color::Magenta,
        format!("{:.0}%", swap_pct * 100.0),
    )
    .block(
        Block::default()
            .title(format!(" Swap  {swap_used_gb:.1} / {swap_total_gb:.1} GiB "))
            .borders(Borders::ALL),
    );

    frame.render_widget(mem_gauge, gauge_chunks[0]);
    frame.render_widget(swap_gauge, gauge_chunks[1]);
}

/// Renders a single-row RAM [`SplitGauge`]
/// in `area`; used by the Overview tab.
pub fn draw_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let pct = if app.mem.total > 0 {
        app.mem.used as f64 / app.mem.total as f64
    } else {
        0.0
    };
    let used_gb = app.mem.used as f64 / 1_073_741_824.0;
    let total_gb = app.mem.total as f64 / 1_073_741_824.0;
    let gauge = SplitGauge::new(
        pct,
        usage_color_f64(pct * 100.0),
        format!("{:.0}%", pct * 100.0),
    )
    .block(
        Block::default()
            .title(format!(
                " RAM  {:.1}/{:.1} GiB  ({:.1}%) ",
                used_gb,
                total_gb,
                pct * 100.0
            ))
            .borders(Borders::ALL),
    );
    frame.render_widget(gauge, area);
}
