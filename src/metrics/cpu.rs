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

//! CPU utilisation metrics.
//!
//! Tracks per-core usage percentages and a 60-second rolling history derived
//! from [`sysinfo::System`].

use sysinfo::System;

use super::{push_history, HISTORY_LEN};

/// Snapshot of CPU utilisation for all logical cores plus a global average.
#[derive(Debug)]
pub struct CpuState {
    /// Per-core CPU usage percentages in the most recent sample (0.0–100.0).
    pub usages: Vec<f32>,
    /// Per-core rolling 60-sample usage history.
    pub history: Vec<Vec<f32>>,
    /// Global (all-cores average) 60-sample usage history.
    pub global_history: Vec<f32>,
}

impl CpuState {
    /// Creates a zeroed [`CpuState`] sized for `cpu_count` logical cores.
    pub fn new(cpu_count: usize) -> Self {
        Self {
            usages: vec![0.0; cpu_count],
            history: vec![vec![0.0; HISTORY_LEN]; cpu_count],
            global_history: vec![0.0; HISTORY_LEN],
        }
    }
}

/// Updates `state` with the current per-core usage values from `sys` and
/// pushes a new sample into each rolling history buffer.
pub fn refresh(state: &mut CpuState, sys: &System) {
    let cpus = sys.cpus();
    let mut global_sum = 0.0_f32;

    for (i, cpu) in cpus.iter().enumerate() {
        let usage = cpu.cpu_usage();
        global_sum += usage;
        if i < state.usages.len() {
            state.usages[i] = usage;
            push_history(&mut state.history[i], usage);
        }
    }

    let global = if cpus.is_empty() {
        0.0
    } else {
        global_sum / cpus.len() as f32
    };
    push_history(&mut state.global_history, global);
}
