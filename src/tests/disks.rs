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

//! Tests for [`crate::metrics::disks`].
//!
//! Covers: [`DiskState`] field construction and the `refresh` function
//! (smoke test verifying no panic, a non-empty result on a machine with at
//! least one mounted filesystem, and the `used ≤ total` invariant for every
//! returned entry).

use crate::metrics::disks::{refresh, DiskState};

// ── DiskState fields ─────────────────────────────────────────────────────────

/// A manually constructed `DiskState` stores all fields correctly.
#[test]
fn test_disk_state_fields_stored_correctly() {
    let d = DiskState {
        name: "sda1".to_string(),
        total: 512_000_000_000,
        used: 128_000_000_000,
        mount: "/".to_string(),
    };
    assert_eq!(d.name, "sda1");
    assert_eq!(d.total, 512_000_000_000);
    assert_eq!(d.used, 128_000_000_000);
    assert_eq!(d.mount, "/");
}

// ── refresh ──────────────────────────────────────────────────────────────────

/// `refresh` must not panic on the host machine.
#[test]
fn test_disks_refresh_does_not_panic() {
    let mut disks: Vec<DiskState> = Vec::new();
    refresh(&mut disks);
    // no assertion needed — we just verify the function returns
}

/// After `refresh`, at least one disk entry must be present (any test machine
/// has a root file system).
#[test]
fn test_disks_refresh_returns_at_least_one_entry() {
    let mut disks: Vec<DiskState> = Vec::new();
    refresh(&mut disks);
    assert!(!disks.is_empty(), "expected at least one mounted filesystem");
}

/// For every disk entry returned by `refresh`, `used` must not exceed `total`.
#[test]
fn test_disks_refresh_used_lte_total() {
    let mut disks: Vec<DiskState> = Vec::new();
    refresh(&mut disks);
    for d in &disks {
        assert!(
            d.used <= d.total,
            "disk '{}' used ({}) > total ({})",
            d.name,
            d.used,
            d.total
        );
    }
}

/// Every disk entry has a non-empty `name` and a non-empty `mount` path after
/// `refresh`.
#[test]
fn test_disks_refresh_names_and_mounts_non_empty() {
    let mut disks: Vec<DiskState> = Vec::new();
    refresh(&mut disks);
    for d in &disks {
        assert!(!d.name.is_empty(), "disk name must not be empty");
        assert!(!d.mount.is_empty(), "disk mount path must not be empty");
    }
}
