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

//! GPU monitoring data types and vendor dispatch.
//!
//! [`GpuEntry`] is vendor-agnostic. Actual collection is performed by the
//! [`amd`] sub-module (via sysfs) and, when the `nvidia` feature is enabled,
//! by the `nvidia` sub-module (via NVML).

pub mod amd;

#[cfg(feature = "nvidia")]
pub mod nvidia;

use super::HISTORY_LEN;

/// Vendor-agnostic snapshot of a single GPU.
#[derive(Debug)]
pub struct GpuEntry {
    /// Human-readable display name (e.g. `"AMD Radeon RX 7900 XTX"`).
    pub name: String,
    /// 3D/compute engine load (0.0–100.0 %).
    pub utilization: f32,
    /// VRAM currently allocated, in bytes.
    pub mem_used: u64,
    /// Total available VRAM, in bytes.
    pub mem_total: u64,
    /// Die temperature in °C, if the driver exposes it.
    pub temperature: Option<u32>,
    /// Power draw in watts, if the driver exposes it.
    pub power_watts: Option<f32>,
    /// 60-sample rolling history of [`utilization`][GpuEntry::utilization].
    pub util_history: Vec<f32>,
    /// 60-sample rolling history of VRAM utilisation percentage.
    pub mem_history: Vec<f32>,
}

impl GpuEntry {
    /// Creates a zeroed [`GpuEntry`] with the given display `name`.
    pub fn new(name: String) -> Self {
        Self {
            name,
            utilization: 0.0,
            mem_used: 0,
            mem_total: 0,
            temperature: None,
            power_watts: None,
            util_history: vec![0.0; HISTORY_LEN],
            mem_history: vec![0.0; HISTORY_LEN],
        }
    }
}
