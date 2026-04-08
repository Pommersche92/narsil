use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};

use crate::app::App;
use crate::ui::helpers::{scroll_indicator, usage_color_f64};
use crate::ui::widgets::SplitGauge;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    const ITEM_H: u16 = 3;

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
        frame.render_widget(gauge, rows[i]);
    }
}
