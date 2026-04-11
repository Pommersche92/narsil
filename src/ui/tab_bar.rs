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

/// Renders the tab-navigation bar in `area`, highlighting the tab at index
/// [`App::selected_tab`].
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.t;

    #[cfg(target_os = "linux")]
    let titles_raw = vec![
        format!("{} [1]", t.tab_overview),
        format!("{} [2]", t.tab_cpu),
        format!("{} [3]", t.tab_memory),
        format!("{} [4]", t.tab_network),
        format!("{} [5]", t.tab_disks),
        format!("{} [6]", t.tab_processes),
        format!("{} [7]", t.tab_gpu),
    ];

    #[cfg(not(target_os = "linux"))]
    let titles_raw = vec![
        format!("{} [1]", t.tab_overview),
        format!("{} [2]", t.tab_cpu),
        format!("{} [3]", t.tab_memory),
        format!("{} [4]", t.tab_network),
        format!("{} [5]", t.tab_disks),
        format!("{} [6]", t.tab_processes),
    ];

    let titles: Vec<Line> = titles_raw
        .into_iter()
        .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::White))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", t.menu_title),
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
