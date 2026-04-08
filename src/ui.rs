use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, Tabs, Widget,
    },
};

use crate::app::{App, GpuEntry, HISTORY_LEN};

const TAB_TITLES: &[&str] = &["Overview [1]", "CPU [2]", "Memory [3]", "Network [4]", "Disks [5]", "Processes [6]", "GPU [7]"];

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_tabs(frame, app, chunks[0]);

    match app.selected_tab {
        0 => draw_overview(frame, app, chunks[1]),
        1 => draw_cpu(frame, app, chunks[1]),
        2 => draw_memory(frame, app, chunks[1]),
        3 => draw_network(frame, app, chunks[1]),
        4 => draw_disks(frame, app, chunks[1]),
        5 => draw_processes(frame, app, chunks[1]),
        6 => draw_gpu(frame, app, chunks[1]),
        _ => {}
    }

    draw_statusbar(frame, app, chunks[2]);
}

fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let inv = Style::default().add_modifier(Modifier::REVERSED);
    let inv_bold = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);

    // Build binding tuples: (keys_label, action)
    let mut bindings: Vec<(&str, &str)> = vec![
        ("Tab / → / l", "Navigate tabs right"),
        ("Shift+Tab / ← / h", "Navigate tabs left"),
        ("1-7", "Jump to tab"),
        ("q / Ctrl-C", "Quit"),
    ];

    let scroll_bindings: &[(&str, &str)] = &[
        ("↑ / k", "Scroll up"),
        ("↓ / j", "Scroll down")
    ];

    if matches!(app.selected_tab, 4 | 5 | 6) {
        bindings.extend_from_slice(scroll_bindings);
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

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = TAB_TITLES
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(
                    " MENU ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .select(app.selected_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, area);
}

// ─── Overview ────────────────────────────────────────────────────────────────

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    draw_cpu_gauge(frame, app, chunks[0]);
    draw_mem_gauge(frame, app, chunks[1]);
    draw_net_sparklines(frame, app, chunks[2]);
    // 2 border rows + 1 header row = 3 rows overhead
    let proc_limit = (chunks[3].height as usize).saturating_sub(3);
    draw_top_processes_table(frame, app, chunks[3], proc_limit, " Processes (sorted by CPU) ");
}

// ─── CPU tab ─────────────────────────────────────────────────────────────────

fn draw_cpu(frame: &mut Frame, app: &App, area: Rect) {
    let cpu_count = app.cpu.usages.len();

    // Global history chart on top
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_cpu_chart(frame, app, chunks[0]);

    // Per-core gauges in a grid
    let cols = 4.min(cpu_count);
    let rows = (cpu_count + cols - 1) / cols;
    let row_constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Length(3))
        .collect();
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
            let color = usage_color(usage);
            let gauge = SplitGauge::new(
                (usage / 100.0) as f64,
                color,
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

fn draw_cpu_chart(frame: &mut Frame, app: &App, area: Rect) {
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

// ─── Memory tab ──────────────────────────────────────────────────────────────

fn draw_memory(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Chart
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

    // Stats
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

// ─── Network tab ─────────────────────────────────────────────────────────────

fn draw_network(frame: &mut Frame, app: &App, area: Rect) {
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

// ─── Disks tab ────────────────────────────────────────────────────────────────

fn draw_disks(frame: &mut Frame, app: &App, area: Rect) {
    const ITEM_H: u16 = 3;

    // Peek inner height without rendering yet
    let inner_h = Block::default().borders(Borders::ALL).inner(area).height;
    let visible = (inner_h / ITEM_H) as usize;
    let total = app.disks.len();
    let scroll = app.disk_scroll;

    let can_up = scroll > 0;
    let can_down = scroll + visible < total;
    let indicator = scroll_indicator(can_up, can_down);
    let title = format!(" Disk Usage{indicator} ");

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.disks.is_empty() {
        return;
    }

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

    for (i, disk) in app.disks.iter().skip(scroll).take(count).enumerate() {
        let pct = if disk.total > 0 {
            disk.used as f64 / disk.total as f64
        } else {
            0.0
        };
        let used_gb = disk.used as f64 / 1_073_741_824.0;
        let total_gb = disk.total as f64 / 1_073_741_824.0;
        let gauge = SplitGauge::new(
            pct,
            usage_color_f64(pct * 100.0),
            format!("{:.0}%", pct * 100.0),
        )
        .block(
            Block::default()
                .title(format!(
                    " {}  {}  {:.1}/{:.1} GiB ",
                    disk.name, disk.mount, used_gb, total_gb
                ))
                .borders(Borders::ALL),
        );
        // rows has count slots + one Min(0) tail
        frame.render_widget(gauge, rows[i]);
    }
}

// ─── Processes tab ────────────────────────────────────────────────────────────

fn draw_processes(frame: &mut Frame, app: &App, area: Rect) {
    // 2 border rows + 1 header row = 3 rows overhead
    let visible = (area.height as usize).saturating_sub(3);
    let total = app.processes.len();
    let scroll = app.process_scroll;
    let can_up = scroll > 0;
    let can_down = scroll + visible < total;
    let indicator = scroll_indicator(can_up, can_down);
    let title = format!(" Processes (sorted by CPU){indicator} ");
    draw_top_processes_table(frame, app, area, visible, &title);
}

fn draw_top_processes_table(frame: &mut Frame, app: &App, area: Rect, limit: usize, title: &str) {
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
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, area);
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn draw_cpu_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let global_cpu = if app.cpu.global_history.is_empty() {
        0.0
    } else {
        *app.cpu.global_history.last().unwrap()
    };
    let color = usage_color(global_cpu);
    let gauge = SplitGauge::new(
        (global_cpu / 100.0) as f64,
        color,
        format!("{:.0}%", global_cpu),
    )
    .block(
        Block::default()
            .title(format!(" CPU  {:.1}% ", global_cpu))
            .borders(Borders::ALL),
    );
    frame.render_widget(gauge, area);
}

fn draw_mem_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let pct = if app.mem.total > 0 {
        app.mem.used as f64 / app.mem.total as f64
    } else {
        0.0
    };
    let used_gb = app.mem.used as f64 / 1_073_741_824.0;
    let total_gb = app.mem.total as f64 / 1_073_741_824.0;
    let color = usage_color_f64(pct * 100.0);
    let gauge = SplitGauge::new(
        pct,
        color,
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

fn draw_net_sparklines(frame: &mut Frame, app: &App, area: Rect) {
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

    let rx_max = rx_data
        .iter()
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);
    let tx_max = tx_data
        .iter()
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);

    let rx_chart = Chart::new(vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&rx_data)])
    .block(
        Block::default()
            .title(format!(
                " ▼ RX {}/s ",
                format_bytes(app.net.rx_bytes_sec)
            ))
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
            .title(format!(
                " ▲ TX {}/s ",
                format_bytes(app.net.tx_bytes_sec)
            ))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(Axis::default().bounds([0.0, tx_max]));

    frame.render_widget(rx_chart, chunks[0]);
    frame.render_widget(tx_chart, chunks[1]);
}

fn scroll_indicator(can_up: bool, can_down: bool) -> &'static str {
    match (can_up, can_down) {
        (true, true) => " ▲▼",
        (true, false) => " ▲",
        (false, true) => " ▼",
        (false, false) => "",
    }
}

// ─── GPU tab ─────────────────────────────────────────────────────────────────

fn draw_gpu(frame: &mut Frame, app: &App, area: Rect) {
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
        draw_gpu_card(frame, gpu, rows[i]);
    }
}

fn draw_gpu_card(frame: &mut Frame, gpu: &GpuEntry, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", gpu.name))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Top: charts side by side | Bottom: two gauges + stats
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
            .title(format!(
                " VRAM  {:.1}/{:.1} GiB ",
                mem_used_gb, mem_total_gb
            ))
            .borders(Borders::ALL),
    )
    .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
    .y_axis(
        Axis::default()
            .bounds([0.0, 100.0])
            .labels(vec![Span::raw("0%"), Span::raw("100%")]),
    );
    frame.render_widget(mem_chart, chart_areas[1]);

    // Bottom row: util gauge | vram gauge | stats
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

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ─── SplitGauge ──────────────────────────────────────────────────────────────
//
// A custom Gauge that inverts the text color per-character at the fill boundary
// so the label is always readable regardless of the bar length.

struct SplitGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    bar_color: Color,
    label: String,
}

impl<'a> SplitGauge<'a> {
    fn new(ratio: f64, bar_color: Color, label: impl Into<String>) -> Self {
        Self {
            block: None,
            ratio: ratio.clamp(0.0, 1.0),
            bar_color,
            label: label.into(),
        }
    }

    fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SplitGauge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let width = inner.width as usize;
        let fill = ((self.ratio * width as f64).round() as usize).min(width);

        // Build per-column character array (spaces + centered label)
        let label_chars: Vec<char> = self.label.chars().collect();
        let label_len = label_chars.len();
        let label_start = width.saturating_sub(label_len) / 2;
        let mut chars = vec![' '; width];
        for (i, &c) in label_chars.iter().enumerate() {
            let pos = label_start + i;
            if pos < width {
                chars[pos] = c;
            }
        }

        // Render on the vertical center row
        let y = inner.y + inner.height / 2;
        for col in 0..width {
            let x = inner.x + col as u16;
            let style = if col < fill {
                // Inside filled bar: invert (dark text on colored background)
                Style::default().fg(Color::Black).bg(self.bar_color)
            } else {
                // Outside bar: colored text on default background
                Style::default().fg(self.bar_color)
            };
            let mut s = [0u8; 4];
            buf.set_string(x, y, chars[col].encode_utf8(&mut s), style);
        }
    }
}

fn usage_color(pct: f32) -> Color {
    usage_color_f64(pct as f64)
}

fn usage_color_f64(pct: f64) -> Color {
    if pct >= 80.0 {
        Color::Red
    } else if pct >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
