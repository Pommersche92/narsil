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

//! Tests for [`crate::ui::widgets::SplitGauge`].
//!
//! [`SplitGauge`] is rendered directly into a [`Buffer`] (no terminal
//! required). Tests verify:
//!
//! * `ratio` clamping above 1.0 and below 0.0,
//! * that filled cells use `bar_color` as background and [`Color::Black`] as
//!   foreground,
//! * that unfilled cells use `bar_color` as foreground and the default
//!   background,
//! * the correct fill/unfill boundary at ratio = 0.5,
//! * that the label is rendered centred across the bar,
//! * that using a block confines the gauge to the block's inner area,
//! * and that a zero-size area does not cause a panic.

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::{Block, Borders, Widget},
};

use crate::ui::widgets::SplitGauge;

// ── ratio clamping ────────────────────────────────────────────────────────────

/// A ratio greater than 1.0 is clamped to 1.0: every cell must be filled.
#[test]
fn test_ratio_clamped_above_one() {
    let area = Rect::new(0, 0, 6, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(2.0, Color::Green, "").render(area, &mut buf);
    for x in 0..6u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_eq!(cell.bg, Color::Green, "x={x} should be filled (bg=Green)");
    }
}

/// A ratio less than 0.0 is clamped to 0.0: no cell must be filled.
#[test]
fn test_ratio_clamped_below_zero() {
    let area = Rect::new(0, 0, 6, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(-1.0, Color::Red, "").render(area, &mut buf);
    for x in 0..6u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_ne!(cell.bg, Color::Red, "x={x} should not be filled (bg≠Red)");
    }
}

// ── fill coverage ─────────────────────────────────────────────────────────────

/// With ratio 1.0, every cell in the bar area has `bg = bar_color` and
/// `fg = Color::Black`.
#[test]
fn test_full_fill_all_cells_inverted() {
    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(1.0, Color::Cyan, "").render(area, &mut buf);
    for x in 0..10u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_eq!(cell.bg, Color::Cyan, "x={x}: expected filled bg");
        assert_eq!(cell.fg, Color::Black, "x={x}: expected filled fg");
    }
}

/// With ratio 0.0, no cell has `bg = bar_color`; all cells have `fg = bar_color`.
#[test]
fn test_empty_fill_no_cells_inverted() {
    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(0.0, Color::Yellow, "").render(area, &mut buf);
    for x in 0..10u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_ne!(cell.bg, Color::Yellow, "x={x} should not be filled");
        assert_eq!(cell.fg, Color::Yellow, "x={x} should use bar_color as fg");
    }
}

/// With ratio 0.5 and width 10, the first 5 cells (0–4) are filled and the
/// last 5 cells (5–9) are unfilled.
#[test]
fn test_half_fill_correct_boundary() {
    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(0.5, Color::Magenta, "").render(area, &mut buf);

    for x in 0..5u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_eq!(cell.bg, Color::Magenta, "x={x} should be filled");
    }
    for x in 5..10u16 {
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_ne!(cell.bg, Color::Magenta, "x={x} should be unfilled");
    }
}

// ── label rendering ───────────────────────────────────────────────────────────

/// The label is centred across the bar width. For a 10-wide gauge and a
/// 3-character label `"42%"`, the label starts at column 3 ( = (10−3)/2 ).
#[test]
fn test_label_is_centred() {
    let label = "42%";
    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(0.0, Color::Green, label).render(area, &mut buf);

    // label_start = (10 - 3) / 2 = 3
    let expected_chars = ['4', '2', '%'];
    for (i, &ch) in expected_chars.iter().enumerate() {
        let x = 3 + i as u16;
        let cell = buf.cell(Position { x, y: 0 }).unwrap();
        assert_eq!(
            cell.symbol(),
            ch.to_string(),
            "column {x} should contain '{ch}'"
        );
    }
}

// ── block integration ─────────────────────────────────────────────────────────

/// When a block with `Borders::ALL` is attached, the gauge fills its inner
/// area. Border cells must not be coloured with `bar_color`.
#[test]
fn test_with_block_gauge_stays_inside_border() {
    // outer 12×3, inner 10×1 at (1, 1)
    let area = Rect::new(0, 0, 12, 3);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(1.0, Color::Blue, "")
        .block(Block::default().borders(Borders::ALL))
        .render(area, &mut buf);

    // Inner cells at y=1, x=1..=10 must all be filled.
    for x in 1..=10u16 {
        let cell = buf.cell(Position { x, y: 1 }).unwrap();
        assert_eq!(cell.bg, Color::Blue, "inner cell ({x},1) should be filled");
    }

    // Top and bottom border rows must NOT be filled.
    for x in 0..12u16 {
        for y in [0u16, 2u16] {
            let cell = buf.cell(Position { x, y }).unwrap();
            assert_ne!(
                cell.bg,
                Color::Blue,
                "border cell ({x},{y}) must not be filled"
            );
        }
    }
}

// ── edge cases ────────────────────────────────────────────────────────────────

/// Rendering into a zero-width area must not panic (early return guard).
#[test]
fn test_zero_width_does_not_panic() {
    let area = Rect::new(0, 0, 0, 1);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(0.5, Color::Green, "50%").render(area, &mut buf);
}

/// Rendering into a zero-height area must not panic (early return guard).
#[test]
fn test_zero_height_does_not_panic() {
    let area = Rect::new(0, 0, 10, 0);
    let mut buf = Buffer::empty(area);
    SplitGauge::new(0.5, Color::Green, "50%").render(area, &mut buf);
}
