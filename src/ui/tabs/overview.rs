use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::App;
use super::{cpu, memory, network, processes};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    cpu::draw_gauge(frame, app, chunks[0]);
    memory::draw_gauge(frame, app, chunks[1]);
    network::draw_sparklines(frame, app, chunks[2]);
    let proc_limit = (chunks[3].height as usize).saturating_sub(3);
    processes::draw_table(frame, app, chunks[3], proc_limit, " Processes (sorted by CPU) ");
}
