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

//! Tests for [`crate::ui::helpers`].
//!
//! Covers: `format_bytes` across all magnitude tiers, `usage_color` and
//! `usage_color_f64` threshold boundaries, and every `scroll_indicator`
//! variant.

use ratatui::style::Color;

use crate::ui::helpers::{format_bytes, scroll_indicator, usage_color, usage_color_f64};

// ── format_bytes ─────────────────────────────────────────────────────────────

/// Zero bytes formats as `"0 B"`.
#[test]
fn test_format_bytes_zero() {
    assert_eq!(format_bytes(0), "0 B");
}

/// A single byte formats as `"1 B"`.
#[test]
fn test_format_bytes_one_byte() {
    assert_eq!(format_bytes(1), "1 B");
}

/// 1 023 bytes (one below the KiB boundary) formats with a `" B"` suffix.
#[test]
fn test_format_bytes_below_kib() {
    assert_eq!(format_bytes(1023), "1023 B");
}

/// Exactly 1 KiB (1 024 bytes) formats as `"1.0 KiB"`.
#[test]
fn test_format_bytes_one_kib() {
    assert_eq!(format_bytes(1024), "1.0 KiB");
}

/// 1 536 bytes formats as `"1.5 KiB"`.
#[test]
fn test_format_bytes_fractional_kib() {
    assert_eq!(format_bytes(1536), "1.5 KiB");
}

/// One byte below 1 MiB (1 048 575 B) formats with a `" KiB"` suffix.
#[test]
fn test_format_bytes_below_mib() {
    let s = format_bytes(1_048_575);
    assert!(s.ends_with(" KiB"), "expected KiB suffix, got: {s}");
}

/// Exactly 1 MiB (1 048 576 bytes) formats as `"1.0 MiB"`.
#[test]
fn test_format_bytes_one_mib() {
    assert_eq!(format_bytes(1_048_576), "1.0 MiB");
}

/// Exactly 1 GiB (1 073 741 824 bytes) formats as `"1.0 GiB"`.
#[test]
fn test_format_bytes_one_gib() {
    assert_eq!(format_bytes(1_073_741_824), "1.0 GiB");
}

/// 4.5 GiB formats as `"4.5 GiB"` (spot-check for large values).
#[test]
fn test_format_bytes_fractional_gib() {
    assert_eq!(format_bytes(4_831_838_208), "4.5 GiB");
}

// ── usage_color ──────────────────────────────────────────────────────────────

/// Usage below 50 % maps to [`Color::Green`].
#[test]
fn test_usage_color_low_is_green() {
    assert_eq!(usage_color(0.0), Color::Green);
    assert_eq!(usage_color(25.0), Color::Green);
    assert_eq!(usage_color(49.9), Color::Green);
}

/// Usage at exactly 50 % maps to [`Color::Yellow`].
#[test]
fn test_usage_color_boundary_50_is_yellow() {
    assert_eq!(usage_color(50.0), Color::Yellow);
}

/// Usage between 50 % (inclusive) and 80 % (exclusive) maps to
/// [`Color::Yellow`].
#[test]
fn test_usage_color_mid_is_yellow() {
    assert_eq!(usage_color(65.0), Color::Yellow);
    assert_eq!(usage_color(79.9), Color::Yellow);
}

/// Usage at exactly 80 % maps to [`Color::Red`].
#[test]
fn test_usage_color_boundary_80_is_red() {
    assert_eq!(usage_color(80.0), Color::Red);
}

/// Usage above 80 % maps to [`Color::Red`].
#[test]
fn test_usage_color_high_is_red() {
    assert_eq!(usage_color(95.0), Color::Red);
    assert_eq!(usage_color(100.0), Color::Red);
}

/// `usage_color_f64` produces the same colours as `usage_color` at the same
/// threshold values.
#[test]
fn test_usage_color_f64_matches_f32() {
    assert_eq!(usage_color_f64(0.0), Color::Green);
    assert_eq!(usage_color_f64(50.0), Color::Yellow);
    assert_eq!(usage_color_f64(80.0), Color::Red);
}

// ── scroll_indicator ─────────────────────────────────────────────────────────

/// When both up and down scrolling are possible, the indicator shows `" ▲▼"`.
#[test]
fn test_scroll_indicator_both() {
    assert_eq!(scroll_indicator(true, true), " ▲▼");
}

/// When only upward scrolling is possible, the indicator shows `" ▲"`.
#[test]
fn test_scroll_indicator_up_only() {
    assert_eq!(scroll_indicator(true, false), " ▲");
}

/// When only downward scrolling is possible, the indicator shows `" ▼"`.
#[test]
fn test_scroll_indicator_down_only() {
    assert_eq!(scroll_indicator(false, true), " ▼");
}

/// When no scrolling is possible, the indicator is an empty string.
#[test]
fn test_scroll_indicator_neither() {
    assert_eq!(scroll_indicator(false, false), "");
}
