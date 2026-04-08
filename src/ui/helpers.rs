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

//! Shared UI helper utilities.
//!
//! Pure functions used by multiple tab modules: byte formatting,
//! usage-based colour selection, and the scroll-position indicator string.

use ratatui::style::Color;

/// Formats a byte count as a human-readable string using binary prefixes.
///
/// | Range              | Example output |
/// |--------------------|----------------|
/// | ≥ 1 GiB            | `"1.0 GiB"`    |
/// | ≥ 1 MiB            | `"1.5 MiB"`    |
/// | ≥ 1 KiB            | `"512.0 KiB"`  |
/// | < 1 KiB            | `"42 B"`        |
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

/// Returns a [`Color`] appropriate for a CPU/memory/GPU usage percentage (`f32`).
///
/// - `< 50 %` → [`Color::Green`]
/// - `50 %–79 %` → [`Color::Yellow`]
/// - `≥ 80 %` → [`Color::Red`]
pub fn usage_color(pct: f32) -> Color {
    usage_color_f64(pct as f64)
}

/// Same as [`usage_color`] but accepts the percentage as an `f64`.
pub fn usage_color_f64(pct: f64) -> Color {
    if pct >= 80.0 {
        Color::Red
    } else if pct >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Returns a static scroll-indicator suffix for widget titles.
///
/// | `can_up` | `can_down` | Returns  |
/// |----------|------------|----------|
/// | `true`   | `true`     | `" ▲▼"`  |
/// | `true`   | `false`    | `" ▲"`   |
/// | `false`  | `true`     | `" ▼"`   |
/// | `false`  | `false`    | `""`     |
pub fn scroll_indicator(can_up: bool, can_down: bool) -> &'static str {
    match (can_up, can_down) {
        (true, true) => " ▲▼",
        (true, false) => " ▲",
        (false, true) => " ▼",
        (false, false) => "",
    }
}
