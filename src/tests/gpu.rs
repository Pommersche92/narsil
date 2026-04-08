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

//! Tests for [`crate::metrics::gpu`].
//!
//! Covers: [`GpuEntry::new`] zeroed initial state, correct name storage, and
//! history vector dimensions. Also contains a smoke test for
//! [`crate::metrics::gpu::amd::refresh`] (no AMD hardware is required — if
//! `/sys/class/drm` is absent the function returns immediately without error).

use crate::metrics::{
    gpu::{amd, GpuEntry},
    HISTORY_LEN,
};

// ── GpuEntry::new ────────────────────────────────────────────────────────────

/// The name passed to `new` is stored verbatim.
#[test]
fn test_gpu_entry_new_stores_name() {
    let entry = GpuEntry::new("AMD Radeon RX 7900 XTX".to_string());
    assert_eq!(entry.name, "AMD Radeon RX 7900 XTX");
}

/// All numeric fields are zero after construction.
#[test]
fn test_gpu_entry_new_numeric_fields_are_zero() {
    let entry = GpuEntry::new("test".to_string());
    assert_eq!(entry.utilization, 0.0);
    assert_eq!(entry.mem_used, 0);
    assert_eq!(entry.mem_total, 0);
}

/// Optional sensor fields are `None` after construction (no driver data yet).
#[test]
fn test_gpu_entry_new_optional_fields_are_none() {
    let entry = GpuEntry::new("test".to_string());
    assert!(entry.temperature.is_none(), "temperature should be None initially");
    assert!(entry.power_watts.is_none(), "power_watts should be None initially");
}

/// Both history vectors have exactly `HISTORY_LEN` entries, all initialised
/// to zero.
#[test]
fn test_gpu_entry_new_history_lengths_and_zeroed() {
    let entry = GpuEntry::new("test".to_string());
    assert_eq!(entry.util_history.len(), HISTORY_LEN);
    assert_eq!(entry.mem_history.len(), HISTORY_LEN);
    assert!(entry.util_history.iter().all(|&v| v == 0.0));
    assert!(entry.mem_history.iter().all(|&v| v == 0.0));
}

// ── amd::refresh ─────────────────────────────────────────────────────────────

/// `amd::refresh` must not panic regardless of whether an AMD GPU is present.
/// On machines without amdgpu the function returns early after failing to find
/// `/sys/class/drm/card*/device/gpu_busy_percent`.
#[test]
fn test_amd_refresh_does_not_panic() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    amd::refresh(&mut gpus);
    // Just verify no panic; GPU presence is not guaranteed in CI.
}

/// If `amd::refresh` finds GPU entries, each must satisfy the
/// `used ≤ total` invariant for VRAM.
#[test]
fn test_amd_refresh_mem_used_lte_total() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    amd::refresh(&mut gpus);
    for gpu in &gpus {
        assert!(
            gpu.mem_used <= gpu.mem_total,
            "GPU '{}' mem_used ({}) > mem_total ({})",
            gpu.name,
            gpu.mem_used,
            gpu.mem_total
        );
    }
}

/// If `amd::refresh` finds GPU entries, their utilisation values must lie in
/// the range 0.0–100.0 %.
#[test]
fn test_amd_refresh_utilization_in_valid_range() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    amd::refresh(&mut gpus);
    for gpu in &gpus {
        assert!(
            (0.0..=100.0).contains(&gpu.utilization),
            "GPU '{}' utilization {} is outside [0, 100]",
            gpu.name,
            gpu.utilization
        );
    }
}
