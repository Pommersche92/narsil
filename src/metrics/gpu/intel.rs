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

//! Intel GPU monitoring via the Linux DRM/hwmon sysfs interfaces.
//!
//! Discovers `i915` and `xe` kernel driver managed cards under `/sys/class/drm`,
//! reads utilisation via GT frequency ratio, LMEM for Intel Arc cards,
//! and temperature via hwmon.

use std::{fs, path::Path};

use super::GpuEntry;
use crate::metrics::push_history;

/// Intel PCI Vendor ID.
const INTEL_VENDOR_ID: u32 = 0x8086;

/// Scans `/sys/class/drm` for Intel GPU cards (i915 or xe drivers),
/// re-initialises the `gpus` list when the card count changes, and updates
/// each entry with fresh GPU utilisation, memory usage, temperature, and power draw.
pub fn refresh(gpus: &mut Vec<GpuEntry>) {
    let drm = Path::new("/sys/class/drm");
    if !drm.exists() {
        return;
    }

    let mut card_paths: Vec<_> = match fs::read_dir(drm) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let n = n.to_string_lossy();
                n.starts_with("card") && !n.contains('-')
            })
            .map(|e| e.path())
            .filter(|p| is_intel_gpu(p))
            .collect(),
        Err(_) => return,
    };
    card_paths.sort();

    if card_paths.is_empty() {
        return;
    }

    if gpus.len() != card_paths.len() {
        *gpus = card_paths
            .iter()
            .map(|p| GpuEntry::new(gpu_name(p)))
            .collect();
    }

    for (i, path) in card_paths.iter().enumerate() {
        let dev = path.join("device");

        // Read utilisation via GT frequency ratio
        let utilization = read_utilization(&dev);

        // Read memory (LMEM for Arc, or shared system memory for iGPU)
        let (mem_used, mem_total, mem_is_gtt) = read_memory(&dev);

        let temperature = hwmon_temp(&dev);
        let power_watts = hwmon_power(&dev);

        let gpu = &mut gpus[i];
        gpu.utilization = utilization;
        gpu.mem_used = mem_used;
        gpu.mem_total = mem_total;
        gpu.mem_is_gtt = mem_is_gtt;
        gpu.temperature = temperature;
        gpu.power_watts = power_watts;
        push_history(&mut gpu.util_history, utilization);
        let mem_pct = if mem_total > 0 {
            mem_used as f32 / mem_total as f32 * 100.0
        } else {
            0.0
        };
        push_history(&mut gpu.mem_history, mem_pct);
    }
}

/// Checks if a card path corresponds to an Intel GPU.
/// Looks for Intel vendor ID in uevent and checks for i915 or xe driver.
fn is_intel_gpu(card_path: &Path) -> bool {
    let dev_path = card_path.join("device");

    // Check vendor ID
    if let Ok(uevent) = fs::read_to_string(dev_path.join("uevent")) {
        for line in uevent.lines() {
            if let Some(id) = line.strip_prefix("PCI_ID=") {
                if let Ok(vendor_id) = u32::from_str_radix(id, 16) {
                    if vendor_id != INTEL_VENDOR_ID {
                        return false;
                    }
                    break;
                }
            }
        }
    } else {
        return false;
    }

    // Check for i915 or xe driver by looking for driver-specific files
    // i915: has gt_act_freq_mhz and other GT files
    // xe: has similar structure but under xe driver
    let has_i915 = dev_path.join("gt_act_freq_mhz").exists()
        || dev_path.join("gt_boost_freq_mhz").exists()
        || dev_path.join("gt_max_freq_mhz").exists()
        || dev_path.join("gt_min_freq_mhz").exists()
        || dev_path.join("gt_rpufreq_mhz").exists();

    let has_xe = dev_path.join("gt_act_freq_mhz").exists()
        || dev_path.join("gt_boost_freq_mhz").exists()
        || dev_path.join("gt_max_freq_mhz").exists()
        || dev_path.join("gt_min_freq_mhz").exists()
        || dev_path.join("gt_rpufreq_mhz").exists()
        || dev_path.join("perfmon_gtact").exists();

    has_i915 || has_xe
}

/// Reads GPU utilisation from GT frequency ratio.
/// For i915: utilisation = (gt_act_freq / gt_max_freq) * 100
/// For xe: similar approach using available frequency files.
fn read_utilization(dev_path: &Path) -> f32 {
    // Try to read current and max frequency
    let act_freq = fs::read_to_string(dev_path.join("gt_act_freq_mhz"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let max_freq = fs::read_to_string(dev_path.join("gt_max_freq_mhz"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);

    if max_freq > 0 {
        (act_freq as f32 / max_freq as f32 * 100.0).min(100.0)
    } else {
        0.0
    }
}

/// Reads memory information for Intel GPUs.
/// For Intel Arc (discrete): reads LMEM (local memory)
/// For Intel iGPU (integrated): no dedicated VRAM, mem_is_gtt remains false
///   (shared system memory is not easily queryable via sysfs)
fn read_memory(dev_path: &Path) -> (u64, u64, bool) {
    // Check for LMEM (Intel Arc discrete GPUs)
    // LMEM is exposed via memdev or DPA registers
    // For now, we check if there's dedicated VRAM available
    let lmem_total = fs::read_to_string(dev_path.join("mem_info_lmem_total"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);

    if lmem_total > 0 {
        // Intel Arc with dedicated LMEM
        let lmem_used = fs::read_to_string(dev_path.join("mem_info_lmem_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        (lmem_used, lmem_total, false)
    } else {
        // iGPU - no dedicated VRAM, shared system memory
        // We cannot easily query shared memory usage, so report 0/0
        (0, 0, false)
    }
}

/// Gets the human-readable GPU name.
fn gpu_name(card_path: &Path) -> String {
    // Check for Intel-specific name files
    if let Ok(n) = fs::read_to_string(card_path.join("device/uevent")) {
        for line in n.lines() {
            if let Some(id) = line.strip_prefix("PCI_ID=") {
                // Try to get a more descriptive name
                if let Ok(model) = fs::read_to_string(card_path.join("device/model")) {
                    let model = model.trim();
                    if !model.is_empty() {
                        return format!("Intel {} [{id}]", model.replace('\n', " "));
                    }
                }
                return format!("Intel GPU [{id}]");
            }
        }
    }

    // Fallback to card name
    card_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Intel GPU".to_string())
}

/// Reads temperature from hwmon.
fn hwmon_temp(dev_path: &Path) -> Option<u32> {
    let hwmon = fs::read_dir(dev_path.join("hwmon")).ok()?;
    for entry in hwmon.filter_map(|e| e.ok()) {
        if let Ok(val) = fs::read_to_string(entry.path().join("temp1_input")) {
            if let Ok(millideg) = val.trim().parse::<u32>() {
                return Some(millideg / 1000);
            }
        }
    }
    None
}

/// Reads power draw from hwmon.
fn hwmon_power(dev_path: &Path) -> Option<f32> {
    let hwmon = fs::read_dir(dev_path.join("hwmon")).ok()?;
    for entry in hwmon.filter_map(|e| e.ok()) {
        if let Ok(val) = fs::read_to_string(entry.path().join("power1_average")) {
            if let Ok(uw) = val.trim().parse::<u64>() {
                return Some(uw as f32 / 1_000_000.0);
            }
        }
    }
    None
}