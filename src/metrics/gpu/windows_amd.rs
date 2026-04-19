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

//! AMD GPU monitoring via DXGI on Windows.
//!
//! Uses [`IDXGIFactory1`] to enumerate AMD graphics adapters and
//! [`IDXGIAdapter3`] (DXGI 1.4, Windows 8.1+) to query dedicated video-memory
//! budget and current usage.
//!
//! GPU utilisation, die temperature, and power draw are not exposed by any
//! standard Windows API without the vendor-specific AMD ADL SDK; those fields
//! are left at their default "not available" values.

use windows::{
    core::Interface,
    Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ADAPTER_DESC1, DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
        DXGI_QUERY_VIDEO_MEMORY_INFO, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1,
    },
};

use super::GpuEntry;
use crate::metrics::push_history;

/// AMD PCI Vendor ID.
const AMD_VENDOR_ID: u32 = 0x1002;

/// Enumerates AMD DXGI adapters and updates `gpus` with fresh VRAM metrics.
///
/// Software / WARP adapters are skipped. GPU utilisation, temperature, and
/// power are reported as unavailable (`0.0` / `None`) because Windows exposes
/// no standard API for those values without ADL.
pub fn refresh(gpus: &mut Vec<GpuEntry>) {
    let factory: IDXGIFactory1 = match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut candidates: Vec<(String, Option<IDXGIAdapter3>)> = Vec::new();
    let mut i = 0u32;
    loop {
        let adapter: IDXGIAdapter1 = match unsafe { factory.EnumAdapters1(i) } {
            Ok(a) => a,
            Err(_) => break,
        };
        i += 1;

        let desc: DXGI_ADAPTER_DESC1 = match unsafe { adapter.GetDesc1() } {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Skip software (WARP) adapters – they have no real VRAM.
        // 2 == DXGI_ADAPTER_FLAG_SOFTWARE
        if (desc.Flags & 2) != 0 {
            continue;
        }

        if desc.VendorId != AMD_VENDOR_ID {
            continue;
        }

        let end = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
        let name = String::from_utf16_lossy(&desc.Description[..end]);

        // IDXGIAdapter3 is available on Windows 8.1+ (DXGI 1.4) and needed for
        // QueryVideoMemoryInfo. Fall back gracefully when unavailable.
        let adapter3: Option<IDXGIAdapter3> = adapter.cast::<IDXGIAdapter3>().ok();
        candidates.push((name, adapter3));
    }

    if candidates.is_empty() {
        return;
    }

    if gpus.len() != candidates.len() {
        *gpus = candidates
            .iter()
            .map(|(name, _)| GpuEntry::new(name.clone()))
            .collect();
    }

    for (idx, (_, adapter3)) in candidates.iter().enumerate() {
        let (mem_used, mem_total) = adapter3
            .as_ref()
            .map(query_vram)
            .unwrap_or((0, 0));

        let gpu = &mut gpus[idx];
        gpu.utilization = 0.0;
        gpu.mem_used = mem_used;
        gpu.mem_total = mem_total;
        gpu.mem_is_gtt = false;
        gpu.temperature = None;
        gpu.power_watts = None;
        push_history(&mut gpu.util_history, 0.0);
        let mem_pct = if mem_total > 0 {
            mem_used as f32 / mem_total as f32 * 100.0
        } else {
            0.0
        };
        push_history(&mut gpu.mem_history, mem_pct);
    }
}

/// Queries the local (on-board VRAM) memory budget and current usage for
/// `adapter` using [`IDXGIAdapter3::QueryVideoMemoryInfo`].
///
/// Returns `(used_bytes, budget_bytes)` or `(0, 0)` on failure.
fn query_vram(adapter: &IDXGIAdapter3) -> (u64, u64) {
    let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
    let ok = unsafe {
        adapter
            .QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info)
            .is_ok()
    };
    if ok {
        (info.CurrentUsage, info.Budget)
    } else {
        (0, 0)
    }
}
