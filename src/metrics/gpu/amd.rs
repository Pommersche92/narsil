use std::{fs, path::Path};

use super::GpuEntry;
use crate::metrics::push_history;

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
        let mem_used = fs::read_to_string(dev.join("mem_info_vram_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let mem_total = fs::read_to_string(dev.join("mem_info_vram_total"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let temperature = hwmon_temp(&dev);
        let power_watts = hwmon_power(&dev);

        let gpu = &mut gpus[i];
        gpu.utilization = utilization;
        gpu.mem_used = mem_used;
        gpu.mem_total = mem_total;
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
