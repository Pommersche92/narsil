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

//! Status bar renderer.
//!
//! Renders a reversed-video footer line showing the active keybindings.
//! Scroll-related bindings are only shown for the tabs that support
//! scrolling (Disks, Processes, GPU).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

/// Renders the status bar in `area` using a reversed-video style.
///
/// Shows the keybindings relevant to the active tab; scroll bindings
/// (`↑ / k`, `↓ / j`) are only included for tabs 4 (Disks), 5 (Processes)
/// and 6 (GPU).
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let inv = Style::default().add_modifier(Modifier::REVERSED);
    let inv_bold = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let t = &app.t;

    let mut bindings: Vec<(&str, &str)> = vec![
        ("Tab / → / l", t.nav_right.as_str()),
        ("Shift+Tab / ← / h", t.nav_left.as_str()),
        #[cfg(target_os = "linux")]
        ("1-7", t.jump_to_tab.as_str()),
        #[cfg(not(target_os = "linux"))]
        ("1-6", t.jump_to_tab.as_str()),
        ("q / Ctrl-C", t.quit.as_str()),
    ];

    #[cfg(target_os = "linux")]
    let scrollable_tab = matches!(app.selected_tab, 4 | 5 | 6);
    #[cfg(not(target_os = "linux"))]
    let scrollable_tab = matches!(app.selected_tab, 4 | 5);
    if scrollable_tab {
        bindings.extend_from_slice(&[("↑ / k", t.scroll_up.as_str()), ("↓ / j", t.scroll_down.as_str())]);
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
