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

//! AMD GPU monitoring via the Linux DRM/hwmon sysfs interfaces.
//!
//! Discovers `amdgpu`-managed cards under `/sys/class/drm`, reads utilisation,
//! VRAM, temperature and power from the corresponding device files.

use std::{fs, path::Path};

use super::GpuEntry;
use crate::metrics::push_history;

/// Scans `/sys/class/drm` for `amdgpu`-managed cards, re-initialises the
/// `gpus` list when the card count changes, and updates each entry with fresh
/// GPU busy percent, VRAM usage, temperature, and power draw.
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
            .filter(|p| p.join("device/gpu_busy_percent").exists())
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

        let utilization = fs::read_to_string(dev.join("gpu_busy_percent"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0);

        // Prefer dedicated VRAM (discrete card). When `mem_info_vram_total`
        // is zero the GPU is an APU whose frame buffer lives entirely in
        // system RAM; fall back to the GTT pool and flag it accordingly.
        let vram_total = fs::read_to_string(dev.join("mem_info_vram_total"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let (mem_used, mem_total, mem_is_gtt) = if vram_total > 0 {
            let vram_used = fs::read_to_string(dev.join("mem_info_vram_used"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            (vram_used, vram_total, false)
        } else {
            let gtt_used = fs::read_to_string(dev.join("mem_info_gtt_used"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let gtt_total = fs::read_to_string(dev.join("mem_info_gtt_total"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            (gtt_used, gtt_total, true)
        };

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

fn gpu_name(card_path: &Path) -> String {
    if let Ok(n) = fs::read_to_string(card_path.join("device/product_name")) {
        let n = n.trim();
        if !n.is_empty() {
            return n.to_string();
        }
    }
    if let Ok(uevent) = fs::read_to_string(card_path.join("device/uevent")) {
        for line in uevent.lines() {
            if let Some(id) = line.strip_prefix("PCI_ID=") {
                return format!("AMD GPU [{id}]");
            }
        }
    }
    card_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "AMD GPU".to_string())
}

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
