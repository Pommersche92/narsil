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

//! NVIDIA GPU monitoring via NVML (enabled with the `nvidia` feature flag).
//!
//! Wraps [`nvml_wrapper`] to query device utilisation, VRAM, temperature and
//! power draw for all detected NVIDIA GPUs.

use nvml_wrapper::Nvml;

use super::GpuEntry;
use crate::metrics::push_history;

/// Refreshes GPU metrics using NVML.
/// Returns `true` if at least one NVML device was found and updated.
/// Updates `gpus` with data from all NVML-accessible NVIDIA GPUs.
///
/// Returns `true` when at least one device was found and updated, allowing
/// the caller to skip the AMD sysfs fallback path.
///
/// The NVML handle is temporarily taken from `nvml` and restored afterwards.
pub fn refresh(gpus: &mut Vec<GpuEntry>, nvml: &mut Option<Nvml>) -> bool {
    let nvml_handle = match nvml.take() {
        Some(n) => n,
        None => return false,
    };

    let count = nvml_handle.device_count().unwrap_or(0) as usize;
    if count == 0 {
        *nvml = Some(nvml_handle);
        return false;
    }

    if gpus.len() != count {
        *gpus = (0..count)
            .map(|i| {
                let name = nvml_handle
                    .device_by_index(i as u32)
                    .and_then(|d| d.name())
                    .unwrap_or_else(|_| format!("NVIDIA GPU {i}"));
                GpuEntry::new(name)
            })
            .collect();
    }

    for i in 0..count {
        if let Ok(device) = nvml_handle.device_by_index(i as u32) {
            let util = device
                .utilization_rates()
                .map(|u| u.gpu as f32)
                .unwrap_or(0.0);
            let (mem_used, mem_total) = device
                .memory_info()
                .map(|m| (m.used, m.total))
                .unwrap_or((0, 0));
            let temperature = device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .ok();
            let power_watts = device.power_usage().ok().map(|mw| mw as f32 / 1000.0);

            let gpu = &mut gpus[i];
            gpu.utilization = util;
            gpu.mem_used = mem_used;
            gpu.mem_total = mem_total;
            gpu.temperature = temperature;
            gpu.power_watts = power_watts;
            push_history(&mut gpu.util_history, util);
            let mem_pct = if mem_total > 0 {
                mem_used as f32 / mem_total as f32 * 100.0
            } else {
                0.0
            };
            push_history(&mut gpu.mem_history, mem_pct);
        }
    }

    *nvml = Some(nvml_handle);
    true
}
