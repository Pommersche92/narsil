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

//! Tests for [`crate::metrics::push_history`].
//!
//! Covers: single-item push, growth up to capacity, eviction of the oldest
//! sample at the capacity boundary, and stable length after overflow.

use crate::metrics::{push_history, HISTORY_LEN};

/// Pushing a single value into an empty vector produces a one-element vector.
#[test]
fn test_push_single_item_into_empty() {
    let mut v: Vec<u32> = Vec::new();
    push_history(&mut v, 42);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0], 42);
}

/// Successive pushes below the capacity produce a vector of exactly that
/// length without any eviction.
#[test]
fn test_push_grows_to_capacity() {
    let mut v: Vec<f32> = Vec::new();
    for i in 0..HISTORY_LEN {
        push_history(&mut v, i as f32);
    }
    assert_eq!(v.len(), HISTORY_LEN);
    assert_eq!(v[0], 0.0);
    assert_eq!(v[HISTORY_LEN - 1], (HISTORY_LEN - 1) as f32);
}

/// When the buffer is already at `HISTORY_LEN`, the next push evicts the
/// oldest sample and appends the new value at the end.
#[test]
fn test_push_evicts_oldest_when_full() {
    let mut v: Vec<u32> = (0..HISTORY_LEN as u32).collect();
    push_history(&mut v, 999);
    assert_eq!(v.len(), HISTORY_LEN, "length must stay at HISTORY_LEN");
    assert_eq!(v[0], 1, "oldest element (0) must have been evicted");
    assert_eq!(*v.last().unwrap(), 999, "new value must be at the end");
}

/// Pushing several values beyond capacity keeps the length exactly at
/// `HISTORY_LEN` and preserves the correct sliding window of values.
#[test]
fn test_push_maintains_length_after_overflow() {
    let mut v: Vec<i32> = (0..HISTORY_LEN as i32).collect();
    for extra in 0..5_i32 {
        push_history(&mut v, 1000 + extra);
    }
    assert_eq!(v.len(), HISTORY_LEN);
    // The last 5 values of the original window (indices 55-59) plus the 5
    // new values should now be the last 10 elements.
    for (offset, extra) in (0..5_i32).enumerate() {
        assert_eq!(v[HISTORY_LEN - 5 + offset], 1000 + extra);
    }
}

/// `push_history` works correctly with `u64` values (as used by
/// [`crate::metrics::network::NetState`]).
#[test]
fn test_push_u64_values() {
    let mut v: Vec<u64> = Vec::new();
    push_history(&mut v, u64::MAX);
    push_history(&mut v, 0);
    assert_eq!(v[0], u64::MAX);
    assert_eq!(v[1], 0);
}
