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

//! Top-level UI module.
//!
//! The public [`draw`] function is the single entry point called from the main
//! loop. It partitions the terminal area into the tab bar, content area, and
//! status bar, then dispatches to the appropriate tab renderer.

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

/// Renders one complete TUI frame.
///
/// Splits `frame` into a three-row layout (tab bar / content / status bar)
/// and dispatches to the tab renderer selected by [`App::selected_tab`].
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
        #[cfg(target_os = "linux")]
        6 => tabs::gpu::draw(frame, app, chunks[1]),
        _ => {}
    }

    statusbar::draw(frame, app, chunks[2]);
}
