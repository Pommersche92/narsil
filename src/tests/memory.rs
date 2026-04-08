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

//! Tests for [`crate::metrics::memory`].
//!
//! Covers: [`MemState::new`] initial values seeded from a live sysinfo
//! snapshot, and the `refresh` function (smoke test and history-length
//! invariants).

use sysinfo::System;

use crate::metrics::{
    memory::{refresh, MemState},
    HISTORY_LEN,
};

// ── MemState::new ────────────────────────────────────────────────────────────

/// On any real machine, total physical memory must be greater than zero.
#[test]
fn test_mem_state_new_has_nonzero_total() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let state = MemState::new(&sys);
    assert!(state.total > 0, "total RAM must be > 0");
}

/// Used memory at construction time must not exceed total memory.
#[test]
fn test_mem_state_new_used_lte_total() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let state = MemState::new(&sys);
    assert!(
        state.used <= state.total,
        "used ({}) > total ({})",
        state.used,
        state.total
    );
}

/// The initial history vector has exactly `HISTORY_LEN` entries (all zero).
#[test]
fn test_mem_state_new_history_length() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let state = MemState::new(&sys);
    assert_eq!(state.history.len(), HISTORY_LEN);
    assert!(state.history.iter().all(|&v| v == 0.0));
}

// ── refresh ──────────────────────────────────────────────────────────────────

/// `refresh` must not panic and the history length must stay at `HISTORY_LEN`.
#[test]
fn test_mem_refresh_does_not_panic_and_caps_history() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut state = MemState::new(&sys);
    refresh(&mut state, &sys);
    assert_eq!(state.history.len(), HISTORY_LEN);
}

/// After `refresh`, `used` must still be ≤ `total`.
#[test]
fn test_mem_refresh_used_lte_total() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut state = MemState::new(&sys);
    refresh(&mut state, &sys);
    assert!(
        state.used <= state.total,
        "used ({}) > total ({}) after refresh",
        state.used,
        state.total
    );
}
