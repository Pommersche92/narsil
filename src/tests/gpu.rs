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
//! history vector dimensions. Also contains smoke tests for the platform-
//! specific refresh backends:
//! - [`crate::metrics::gpu::amd::refresh`] on Linux (no AMD hardware required)
//! - [`crate::metrics::gpu::intel::refresh`] on Linux (no Intel hardware required)
//! - [`crate::metrics::gpu::windows_amd::refresh`] on Windows (no AMD hardware required)
//! - [`crate::metrics::gpu::windows_intel::refresh`] on Windows (no Intel hardware required)

use crate::metrics::{
    gpu::GpuEntry,
    HISTORY_LEN,
};

#[cfg(target_os = "linux")]
use crate::metrics::gpu::amd;

#[cfg(target_os = "linux")]
use crate::metrics::gpu::intel;

#[cfg(target_os = "windows")]
use crate::metrics::gpu::windows_amd;

#[cfg(target_os = "windows")]
use crate::metrics::gpu::windows_intel;

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

/// `mem_is_gtt` is `false` after construction — assumes dedicated VRAM until
/// the first AMD sysfs refresh determines otherwise.
#[test]
fn test_gpu_entry_new_mem_is_gtt_is_false() {
    let entry = GpuEntry::new("test".to_string());
    assert!(!entry.mem_is_gtt, "mem_is_gtt should be false on construction");
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

// ── amd::refresh (Linux) ─────────────────────────────────────────────────────

/// `amd::refresh` must not panic regardless of whether an AMD GPU is present.
/// On machines without amdgpu the function returns early after failing to find
/// `/sys/class/drm/card*/device/gpu_busy_percent`.
#[test]
#[cfg(target_os = "linux")]
fn test_amd_refresh_does_not_panic() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    amd::refresh(&mut gpus);
    // Just verify no panic; GPU presence is not guaranteed in CI.
}

/// If `amd::refresh` finds GPU entries, each must satisfy the
/// `used ≤ total` invariant for VRAM.
#[test]
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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

// ── windows_amd::refresh (Windows) ───────────────────────────────────────────

/// `windows_amd::refresh` must not panic regardless of whether an AMD GPU is
/// present. On machines without AMD hardware the function returns early after
/// finding no matching DXGI adapters.
#[test]
#[cfg(target_os = "windows")]
fn test_windows_amd_refresh_does_not_panic() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_amd::refresh(&mut gpus);
    // Just verify no panic; GPU presence is not guaranteed in CI.
}

/// If `windows_amd::refresh` finds GPU entries, each must satisfy the
/// `used ≤ total` invariant for VRAM.
#[test]
#[cfg(target_os = "windows")]
fn test_windows_amd_refresh_mem_used_lte_total() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_amd::refresh(&mut gpus);
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

/// `windows_amd::refresh` reports utilisation as 0.0 (DXGI does not expose
/// engine load without ADL).
#[test]
#[cfg(target_os = "windows")]
fn test_windows_amd_refresh_utilization_is_zero() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_amd::refresh(&mut gpus);
    for gpu in &gpus {
        assert_eq!(
            gpu.utilization, 0.0,
            "GPU '{}' utilization should be 0.0 on Windows (DXGI limitation)",
            gpu.name
        );
    }
}

/// `windows_amd::refresh` leaves temperature and power as `None` (DXGI does
/// not expose those without ADL).
#[test]
#[cfg(target_os = "windows")]
fn test_windows_amd_refresh_optional_sensors_are_none() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_amd::refresh(&mut gpus);
    for gpu in &gpus {
        assert!(
            gpu.temperature.is_none(),
            "GPU '{}' temperature should be None on Windows",
            gpu.name
        );
        assert!(
            gpu.power_watts.is_none(),
            "GPU '{}' power_watts should be None on Windows",
            gpu.name
        );
    }
}

// ── intel::refresh (Linux) ───────────────────────────────────────────────────

/// `intel::refresh` must not panic regardless of whether an Intel GPU is present.
/// On machines without i915/xe the function returns early after failing to find
/// Intel GPU cards.
#[test]
#[cfg(target_os = "linux")]
fn test_intel_refresh_does_not_panic() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    intel::refresh(&mut gpus);
    // Just verify no panic; GPU presence is not guaranteed in CI.
}

/// If `intel::refresh` finds GPU entries, each must satisfy the
/// `used ≤ total` invariant for memory.
#[test]
#[cfg(target_os = "linux")]
fn test_intel_refresh_mem_used_lte_total() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    intel::refresh(&mut gpus);
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

/// If `intel::refresh` finds GPU entries, their utilisation values must lie in
/// the range 0.0–100.0 %.
#[test]
#[cfg(target_os = "linux")]
fn test_intel_refresh_utilization_in_valid_range() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    intel::refresh(&mut gpus);
    for gpu in &gpus {
        assert!(
            (0.0..=100.0).contains(&gpu.utilization),
            "GPU '{}' utilization {} is outside [0, 100]",
            gpu.name,
            gpu.utilization
        );
    }
}

// ── windows_intel::refresh (Windows) ──────────────────────────────────────────

/// `windows_intel::refresh` must not panic regardless of whether an Intel GPU is
/// present. On machines without Intel hardware the function returns early after
/// finding no matching DXGI adapters.
#[test]
#[cfg(target_os = "windows")]
fn test_windows_intel_refresh_does_not_panic() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_intel::refresh(&mut gpus);
    // Just verify no panic; GPU presence is not guaranteed in CI.
}

/// If `windows_intel::refresh` finds GPU entries, each must satisfy the
/// `used ≤ total` invariant for VRAM.
#[test]
#[cfg(target_os = "windows")]
fn test_windows_intel_refresh_mem_used_lte_total() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_intel::refresh(&mut gpus);
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

/// `windows_intel::refresh` reports utilisation as 0.0 (DXGI does not expose
/// engine load without Intel tools).
#[test]
#[cfg(target_os = "windows")]
fn test_windows_intel_refresh_utilization_is_zero() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_intel::refresh(&mut gpus);
    for gpu in &gpus {
        assert_eq!(
            gpu.utilization, 0.0,
            "GPU '{}' utilization should be 0.0 on Windows (DXGI limitation)",
            gpu.name
        );
    }
}

/// `windows_intel::refresh` leaves temperature and power as `None` (DXGI does
/// not expose those without Intel tools).
#[test]
#[cfg(target_os = "windows")]
fn test_windows_intel_refresh_optional_sensors_are_none() {
    let mut gpus: Vec<GpuEntry> = Vec::new();
    windows_intel::refresh(&mut gpus);
    for gpu in &gpus {
        assert!(
            gpu.temperature.is_none(),
            "GPU '{}' temperature should be None on Windows",
            gpu.name
        );
        assert!(
            gpu.power_watts.is_none(),
            "GPU '{}' power_watts should be None on Windows",
            gpu.name
        );
    }
}
