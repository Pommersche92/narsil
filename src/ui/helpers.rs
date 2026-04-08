use ratatui::style::Color;

pub fn format_bytes(bytes: u64) -> String {
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

pub fn usage_color(pct: f32) -> Color {
    usage_color_f64(pct as f64)
}

pub fn usage_color_f64(pct: f64) -> Color {
    if pct >= 80.0 {
        Color::Red
    } else if pct >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

pub fn scroll_indicator(can_up: bool, can_down: bool) -> &'static str {
    match (can_up, can_down) {
        (true, true) => " ▲▼",
        (true, false) => " ▲",
        (false, true) => " ▼",
        (false, false) => "",
    }
}
