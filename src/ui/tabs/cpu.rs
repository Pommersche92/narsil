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
use crate::ui::helpers::usage_color;
use crate::ui::widgets::SplitGauge;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let cpu_count = app.cpu.usages.len();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_chart(frame, app, chunks[0]);

    let cols = 4.min(cpu_count);
    let rows = (cpu_count + cols - 1) / cols;
    let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(3)).collect();
    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(chunks[1]);

    for (row_idx, row_area) in row_areas.iter().enumerate() {
        let start = row_idx * cols;
        let end = (start + cols).min(cpu_count);
        let count = end - start;
        let col_constraints: Vec<Constraint> =
            (0..count).map(|_| Constraint::Ratio(1, count as u32)).collect();
        let col_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);

        for (col_idx, col_area) in col_areas.iter().enumerate() {
            let core_idx = start + col_idx;
            let usage = app.cpu.usages[core_idx];
            let gauge = SplitGauge::new(
                (usage / 100.0) as f64,
                usage_color(usage),
                format!("{:.0}%", usage),
            )
            .block(
                Block::default()
                    .title(format!(" CPU{core_idx} "))
                    .borders(Borders::ALL),
            );
            frame.render_widget(gauge, *col_area);
        }
    }
}

pub fn draw_chart(frame: &mut Frame, app: &App, area: Rect) {
    let data: Vec<(f64, f64)> = app
        .cpu
        .global_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let datasets = vec![Dataset::default()
        .name("CPU %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    " CPU Usage History ",
                    Style::default().fg(Color::Cyan),
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

    frame.render_widget(chart, area);
}

pub fn draw_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let global_cpu = app.cpu.global_history.last().copied().unwrap_or(0.0);
    let gauge = SplitGauge::new(
        (global_cpu / 100.0) as f64,
        usage_color(global_cpu),
        format!("{:.0}%", global_cpu),
    )
    .block(
        Block::default()
            .title(format!(" CPU  {:.1}% ", global_cpu))
            .borders(Borders::ALL),
    );
    frame.render_widget(gauge, area);
}
