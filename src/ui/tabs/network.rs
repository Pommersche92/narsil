use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

use crate::app::App;
use crate::metrics::HISTORY_LEN;
use crate::ui::helpers::format_bytes;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let rx_data: Vec<(f64, f64)> = app
        .net
        .rx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();
    let tx_data: Vec<(f64, f64)> = app
        .net
        .tx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let max_val = rx_data
        .iter()
        .chain(tx_data.iter())
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);

    let datasets = vec![
        Dataset::default()
            .name("▼ RX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&rx_data),
        Dataset::default()
            .name("▲ TX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&tx_data),
    ];

    let net_mid_label = format_bytes(max_val as u64 / 2);
    let net_max_label = format_bytes(max_val as u64);
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    " Network I/O History ",
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
                .bounds([0.0, max_val])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(net_mid_label.as_str()),
                    Span::raw(net_max_label.as_str()),
                ]),
        );

    frame.render_widget(chart, chunks[0]);

    let text = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  ▼ RX: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}/s", format_bytes(app.net.rx_bytes_sec))),
        ]),
        Line::from(vec![
            Span::styled("  ▲ TX: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}/s", format_bytes(app.net.tx_bytes_sec))),
        ]),
    ])
    .block(
        Block::default()
            .title(" Current Throughput ")
            .borders(Borders::ALL),
    );

    frame.render_widget(text, chunks[1]);
}

pub fn draw_sparklines(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let rx_data: Vec<(f64, f64)> = app
        .net
        .rx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();
    let tx_data: Vec<(f64, f64)> = app
        .net
        .tx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let rx_max = rx_data.iter().map(|(_, v)| *v).fold(1.0_f64, f64::max);
    let tx_max = tx_data.iter().map(|(_, v)| *v).fold(1.0_f64, f64::max);

    let rx_chart = Chart::new(vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&rx_data)])
    .block(
        Block::default()
            .title(format!(" ▼ RX {}/s ", format_bytes(app.net.rx_bytes_sec)))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(Axis::default().bounds([0.0, rx_max]));

    let tx_chart = Chart::new(vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Yellow))
        .data(&tx_data)])
    .block(
        Block::default()
            .title(format!(" ▲ TX {}/s ", format_bytes(app.net.tx_bytes_sec)))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(Axis::default().bounds([0.0, tx_max]));

    frame.render_widget(rx_chart, chunks[0]);
    frame.render_widget(tx_chart, chunks[1]);
}
