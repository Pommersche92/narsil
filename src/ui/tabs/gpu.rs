use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

use crate::app::App;
use crate::metrics::{GpuEntry, HISTORY_LEN};
use crate::ui::helpers::{scroll_indicator, usage_color};
use crate::ui::widgets::SplitGauge;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    const ITEM_H: u16 = 12;

    if app.gpus.is_empty() {
        let msg = Paragraph::new(
            "No compatible GPU detected.\n\
             • AMD:    requires amdgpu kernel driver\n\
             • NVIDIA: rebuild with: cargo build --features nvidia",
        )
        .block(Block::default().title(" GPU ").borders(Borders::ALL));
        frame.render_widget(msg, area);
        return;
    }

    let inner_h = Block::default().borders(Borders::ALL).inner(area).height;
    let visible = ((inner_h / ITEM_H) as usize).max(1);
    let total = app.gpus.len();
    let scroll = app.gpu_scroll;

    let can_up = scroll > 0;
    let can_down = scroll + visible < total;
    let indicator = scroll_indicator(can_up, can_down);
    let title = format!(" GPU{indicator} ");

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let count = visible.min(total.saturating_sub(scroll));
    if count == 0 {
        return;
    }

    let constraints: Vec<Constraint> = (0..count)
        .map(|_| Constraint::Length(ITEM_H))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, gpu) in app.gpus.iter().skip(scroll).take(count).enumerate() {
        draw_card(frame, gpu, rows[i]);
    }
}

pub fn draw_card(frame: &mut Frame, gpu: &GpuEntry, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", gpu.name))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(inner);

    let chart_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[0]);

    // Utilisation history
    let util_data: Vec<(f64, f64)> = gpu
        .util_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();
    let util_chart = Chart::new(vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&util_data)])
    .block(
        Block::default()
            .title(format!(" GPU  {:.0}% ", gpu.utilization))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(
        Axis::default()
            .bounds([0.0, 100.0])
            .labels(vec![Span::raw("0%"), Span::raw("100%")]),
    );
    frame.render_widget(util_chart, chart_areas[0]);

    // VRAM history
    let mem_data: Vec<(f64, f64)> = gpu
        .mem_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();
    let mem_used_gb = gpu.mem_used as f64 / 1_073_741_824.0;
    let mem_total_gb = gpu.mem_total as f64 / 1_073_741_824.0;
    let mem_chart = Chart::new(vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Magenta))
        .data(&mem_data)])
    .block(
        Block::default()
            .title(format!(" VRAM  {:.1}/{:.1} GiB ", mem_used_gb, mem_total_gb))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(
        Axis::default()
            .bounds([0.0, 100.0])
            .labels(vec![Span::raw("0%"), Span::raw("100%")]),
    );
    frame.render_widget(mem_chart, chart_areas[1]);

    // Bottom row: util gauge | VRAM gauge | stats
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(sections[1]);

    let util_gauge = SplitGauge::new(
        (gpu.utilization / 100.0) as f64,
        usage_color(gpu.utilization),
        format!("{:.0}%", gpu.utilization),
    )
    .block(Block::default().title(" Util ").borders(Borders::ALL));
    frame.render_widget(util_gauge, bottom[0]);

    let mem_pct = if gpu.mem_total > 0 {
        gpu.mem_used as f64 / gpu.mem_total as f64
    } else {
        0.0
    };
    let mem_gauge = SplitGauge::new(
        mem_pct,
        Color::Magenta,
        format!("{:.0}%", mem_pct * 100.0),
    )
    .block(Block::default().title(" VRAM ").borders(Borders::ALL));
    frame.render_widget(mem_gauge, bottom[1]);

    let temp_str = gpu
        .temperature
        .map(|t| format!("{t}°C"))
        .unwrap_or_else(|| "N/A".into());
    let power_str = gpu
        .power_watts
        .map(|p| format!("{p:.0}W"))
        .unwrap_or_else(|| "N/A".into());
    let stats = Paragraph::new(Line::from(vec![
        Span::styled(" Temp: ", Style::default().fg(Color::Yellow)),
        Span::raw(temp_str),
        Span::styled("  Pow: ", Style::default().fg(Color::Yellow)),
        Span::raw(power_str),
    ]))
    .block(Block::default().title(" Stats ").borders(Borders::ALL));
    frame.render_widget(stats, bottom[2]);
}
