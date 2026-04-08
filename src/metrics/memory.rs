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

//! RAM and swap metrics.
//!
//! Records current usage in bytes and a 60-second history of utilisation
//! percentage, derived from [`sysinfo::System`].

use sysinfo::System;

use super::{push_history, HISTORY_LEN};

/// Snapshot of RAM and swap utilisation.
#[derive(Debug)]
pub struct MemState {
    /// Total physical memory in bytes.
    pub total: u64,
    /// Currently used physical memory in bytes.
    pub used: u64,
    /// Total swap space in bytes.
    pub swap_total: u64,
    /// Currently used swap space in bytes.
    pub swap_used: u64,
    /// 60-sample rolling history of RAM utilisation percentage (0.0–100.0).
    pub history: Vec<f32>,
}

impl MemState {
    /// Creates a [`MemState`] seeded with values from the first `sys` snapshot.
    pub fn new(sys: &System) -> Self {
        Self {
            total: sys.total_memory(),
            used: sys.used_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            history: vec![0.0; HISTORY_LEN],
        }
    }
}

/// Updates `state` from the latest `sys` snapshot and appends a utilisation
/// percentage sample to the rolling history.
pub fn refresh(state: &mut MemState, sys: &System) {
    state.total = sys.total_memory();
    state.used = sys.used_memory();
    state.swap_total = sys.total_swap();
    state.swap_used = sys.used_swap();

    let pct = if state.total > 0 {
        state.used as f32 / state.total as f32 * 100.0
    } else {
        0.0
    };
    push_history(&mut state.history, pct);
}
