pub mod helpers;
pub mod statusbar;
pub mod tab_bar;
pub mod tabs;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;

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

    tab_bar::draw(frame, app, chunks[0]);

    match app.selected_tab {
        0 => tabs::overview::draw(frame, app, chunks[1]),
        1 => tabs::cpu::draw(frame, app, chunks[1]),
        2 => tabs::memory::draw(frame, app, chunks[1]),
        3 => tabs::network::draw(frame, app, chunks[1]),
        4 => tabs::disks::draw(frame, app, chunks[1]),
        5 => tabs::processes::draw(frame, app, chunks[1]),
        6 => tabs::gpu::draw(frame, app, chunks[1]),
        _ => {}
    }

    statusbar::draw(frame, app, chunks[2]);
}
