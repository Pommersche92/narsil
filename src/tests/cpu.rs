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

//! Tests for [`crate::metrics::cpu`].
//!
//! Covers: [`CpuState::new`] dimension invariants (sizes of all inner vectors),
//! initial zeroed values, and the `refresh` function (live OS call: checks
//! that it does not panic, that history length is preserved, and that reported
//! usages fall in the valid 0–100 % range).

use sysinfo::System;

use crate::metrics::{
    cpu::{refresh, CpuState},
    HISTORY_LEN,
};

// ── CpuState::new ────────────────────────────────────────────────────────────

/// `new(n)` produces a `usages` vector of exactly `n` elements.
#[test]
fn test_cpu_state_new_usages_length() {
    let state = CpuState::new(8);
    assert_eq!(state.usages.len(), 8);
}

/// `new(n)` produces a `history` outer vector of exactly `n` elements.
#[test]
fn test_cpu_state_new_history_outer_length() {
    let state = CpuState::new(4);
    assert_eq!(state.history.len(), 4);
}

/// Each per-core history slice has exactly `HISTORY_LEN` entries.
#[test]
fn test_cpu_state_new_history_inner_length() {
    let state = CpuState::new(4);
    for (i, core_hist) in state.history.iter().enumerate() {
        assert_eq!(
            core_hist.len(),
            HISTORY_LEN,
            "core {i} history should have HISTORY_LEN entries"
        );
    }
}

/// The global history has exactly `HISTORY_LEN` entries.
#[test]
fn test_cpu_state_new_global_history_length() {
    let state = CpuState::new(4);
    assert_eq!(state.global_history.len(), HISTORY_LEN);
}

/// All initial usages are zero.
#[test]
fn test_cpu_state_new_usages_all_zero() {
    let state = CpuState::new(4);
    assert!(state.usages.iter().all(|&u| u == 0.0));
}

/// All initial history samples are zero.
#[test]
fn test_cpu_state_new_history_all_zero() {
    let state = CpuState::new(2);
    assert!(state.history.iter().flatten().all(|&v| v == 0.0));
    assert!(state.global_history.iter().all(|&v| v == 0.0));
}

// ── refresh ──────────────────────────────────────────────────────────────────

/// `refresh` must not panic on a normal sysinfo snapshot, and the core count
/// must remain unchanged after the call.
#[test]
fn test_cpu_refresh_does_not_panic() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_count = sys.cpus().len().max(1);
    let mut state = CpuState::new(cpu_count);
    refresh(&mut state, &sys);
    assert_eq!(state.usages.len(), cpu_count);
}

/// After `refresh`, the global history length stays at `HISTORY_LEN`.
#[test]
fn test_cpu_refresh_global_history_stays_capped() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_count = sys.cpus().len().max(1);
    let mut state = CpuState::new(cpu_count);
    refresh(&mut state, &sys);
    assert_eq!(state.global_history.len(), HISTORY_LEN);
}

/// All per-core usage values reported after `refresh` lie in the range
/// [0.0, 100.0].
#[test]
fn test_cpu_refresh_usages_in_valid_range() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_count = sys.cpus().len().max(1);
    let mut state = CpuState::new(cpu_count);
    refresh(&mut state, &sys);
    for (i, &usage) in state.usages.iter().enumerate() {
        assert!(
            (0.0..=100.0).contains(&usage),
            "CPU{i} usage {usage} is outside [0, 100]"
        );
    }
}
