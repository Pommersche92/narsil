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

//! Tests for [`crate::metrics::processes`].
//!
//! Covers: [`ProcessEntry`] field construction and the `refresh` function
//! (smoke test, result-count limit, and CPU-descending sort invariant).

use sysinfo::System;

use crate::metrics::processes::{refresh, ProcessEntry};

// ── ProcessEntry fields ───────────────────────────────────────────────────────

/// A manually constructed `ProcessEntry` stores all fields correctly.
#[test]
fn test_process_entry_fields_stored_correctly() {
    let p = ProcessEntry {
        pid: 1234,
        name: "bash".to_string(),
        cpu: 12.5,
        mem_kb: 4096,
    };
    assert_eq!(p.pid, 1234);
    assert_eq!(p.name, "bash");
    assert!((p.cpu - 12.5).abs() < f32::EPSILON);
    assert_eq!(p.mem_kb, 4096);
}

// ── refresh ───────────────────────────────────────────────────────────────────

/// `refresh` must not panic on a normal sysinfo snapshot.
#[test]
fn test_processes_refresh_does_not_panic() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut processes: Vec<ProcessEntry> = Vec::new();
    refresh(&mut processes, &sys);
}

/// The result list must contain at most 100 entries regardless of how many
/// processes are running.
#[test]
fn test_processes_refresh_count_within_limit() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut processes: Vec<ProcessEntry> = Vec::new();
    refresh(&mut processes, &sys);
    assert!(
        processes.len() <= 100,
        "refresh should keep at most 100 processes, got {}",
        processes.len()
    );
}

/// After `refresh`, the process list must be sorted by CPU usage in
/// descending order (highest utilisation first).
#[test]
fn test_processes_refresh_sorted_by_cpu_descending() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut processes: Vec<ProcessEntry> = Vec::new();
    refresh(&mut processes, &sys);
    for i in 0..processes.len().saturating_sub(1) {
        assert!(
            processes[i].cpu >= processes[i + 1].cpu,
            "processes not sorted: processes[{i}].cpu ({:.2}) < processes[{}].cpu ({:.2})",
            processes[i].cpu,
            i + 1,
            processes[i + 1].cpu
        );
    }
}

/// Every process entry must have a non-empty name after `refresh`.
#[test]
fn test_processes_refresh_names_non_empty() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut processes: Vec<ProcessEntry> = Vec::new();
    refresh(&mut processes, &sys);
    for p in &processes {
        assert!(!p.name.is_empty(), "PID {} has an empty name", p.pid);
    }
}
