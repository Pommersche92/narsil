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

//! Tab bar renderer.
//!
//! Draws the top navigation bar containing all tab titles and highlights the
//! currently active tab.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};

use crate::app::App;

#[cfg(target_os = "linux")]
const TAB_TITLES: &[&str] = &[
    "Overview [1]",
    "CPU [2]",
    "Memory [3]",
    "Network [4]",
    "Disks [5]",
    "Processes [6]",
    "GPU [7]",
];

#[cfg(not(target_os = "linux"))]
const TAB_TITLES: &[&str] = &[
    "Overview [1]",
    "CPU [2]",
    "Memory [3]",
    "Network [4]",
    "Disks [5]",
    "Processes [6]",
];

/// Renders the tab-navigation bar in `area`, highlighting the tab at index
/// [`App::selected_tab`].
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
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
