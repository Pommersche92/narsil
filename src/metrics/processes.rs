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

//! Process list metrics.
//!
//! Gathers all running processes, sorts them by CPU usage descending, and
//! truncates the list to the top 100 entries.

use sysinfo::System;

/// Lightweight snapshot of a single process.
#[derive(Debug)]
pub struct ProcessEntry {
    /// Operating-system process identifier.
    pub pid: u32,
    /// Process name as reported by the OS (may be truncated).
    pub name: String,
    /// CPU usage percentage at the time of sampling (summed across all cores).
    pub cpu: f32,
    /// Resident memory in kibibytes.
    pub mem_kb: u64,
}

/// Rebuilds `processes` from the current `sys` snapshot, sorts by CPU usage
/// descending, and retains the top 100 entries.
pub fn refresh(processes: &mut Vec<ProcessEntry>, sys: &System) {
    let mut procs: Vec<ProcessEntry> = sys
        .processes()
        .values()
        .map(|p| ProcessEntry {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu: p.cpu_usage(),
            mem_kb: p.memory() / 1024,
        })
        .collect();

    procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(100);
    *processes = procs;
}
