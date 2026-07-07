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

//! Intel GPU monitoring on macOS via IOKit.
//!
//! Uses IOKit to query Intel integrated GPU information.
//! Note: macOS Intel GPU support is limited; utilisation and memory metrics
//! are not easily accessible without additional tools.

use std::fs;

use super::GpuEntry;
use crate::metrics::push_history;

/// Refreshes Intel GPU metrics on macOS.
///
/// Currently, macOS does not expose Intel GPU utilisation or memory usage
/// through standard APIs. This function returns an empty list, but the
/// GPU tab will still be available for other vendors.
pub fn refresh(_gpus: &mut Vec<GpuEntry>) {
    // macOS Intel GPU support is limited.
    // The system_profiler SPDisplaysDataType command can provide some info,
    // but it's not real-time and requires parsing command output.
    // For now, we don't populate Intel GPU data on macOS.
    // The GPU tab will still work for other vendors (AMD/NVIDIA) if present.
}

/// Checks if an Intel GPU is present on macOS.
/// Returns true if the system has Intel integrated graphics.
pub fn has_intel_gpu() -> bool {
    // Check if system_profiler can detect Intel GPU
    // This is a simple heuristic - check for Intel in display info
    if let Ok(output) = std::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
    {
        if let Ok(json) = String::from_utf8(output.stdout) {
            return json.contains("\"vendor\":\"Intel") || json.contains("\"sppci_vendor\":\"0x86");
        }
    }
    false
}