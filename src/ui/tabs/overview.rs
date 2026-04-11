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

//! Overview tab renderer.
//!
//! Combines the CPU gauge, RAM gauge, network sparklines, and a short process
//! table into a single at-a-glance dashboard.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::App;
use super::{cpu, memory, network, processes};

/// Renders the Overview tab in `area`.
///
/// Layout (top to bottom):
/// 1. CPU usage gauge (4 rows)
/// 2. RAM usage gauge (4 rows)
/// 3. Network RX/TX sparklines (7 rows)
/// 4. Top-processes table (remaining space)
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
    let proc_title = format!(" {} ", app.t.processes_title);
    processes::draw_table(frame, app, chunks[3], proc_limit, &proc_title);
}
