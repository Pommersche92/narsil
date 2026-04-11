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

//! Central application state.
//!
//! [`App`] owns every metric sub-state and delegates per-tick data refresh to
//! the domain modules in [`crate::metrics`].

use sysinfo::System;

#[cfg(all(target_os = "linux", feature = "nvidia"))]
use nvml_wrapper::Nvml;

use crate::i18n::{Translations, get_translations};
use crate::metrics::{
    cpu, disks, memory, network, processes,
    CpuState, DiskState, MemState, NetState, ProcessEntry,
};
#[cfg(target_os = "linux")]
use crate::metrics::{gpu as gpu_metrics, GpuEntry};

/// Central application state.
///
/// Owns all metric sub-states, the underlying [`sysinfo::System`] instance,
/// and UI bookkeeping fields such as the active tab and scroll offsets.
pub struct App {
    /// The sysinfo system handle used for CPU, memory and process queries.
    pub sys: System,
    /// CPU utilisation state (per-core usages and rolling history).
    pub cpu: CpuState,
    /// RAM and swap utilisation state.
    pub mem: MemState,
    /// Network throughput state (per-tick RX/TX deltas and rolling history).
    pub net: NetState,
    /// Mounted filesystems with their disk usage.
    pub disks: Vec<DiskState>,
    /// Top-100 processes sorted by CPU usage descending.
    pub processes: Vec<ProcessEntry>,
    /// Detected GPU entries (AMD via sysfs or NVIDIA via NVML).
    #[cfg(target_os = "linux")]
    pub gpus: Vec<GpuEntry>,
    /// Tick interval in milliseconds (default: 1 000 ms / 1 Hz).
    pub tick_rate_ms: u64,
    /// Index of the currently visible tab (0 = Overview … 6 = GPU).
    pub selected_tab: usize,
    /// Scroll offset for the Processes tab.
    pub process_scroll: usize,
    /// Scroll offset for the Disks tab.
    pub disk_scroll: usize,
    /// Scroll offset for the GPU tab.
    #[cfg(target_os = "linux")]
    pub gpu_scroll: usize,
    /// Initialised NVML handle, or `None` if NVML is unavailable at runtime.
    #[cfg(all(target_os = "linux", feature = "nvidia"))]
    pub(crate) nvml: Option<Nvml>,
    /// Active UI translation table.
    pub t: Translations,
}

impl App {
    /// Creates a new [`App`] for the given ISO 639-1 language code, performs
    /// an initial full system refresh, and — when the `nvidia` feature is
    /// enabled — attempts to initialise NVML.
    pub fn new(lang_code: &str) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_count = sys.cpus().len().max(1);

        App {
            cpu: CpuState::new(cpu_count),
            mem: MemState::new(&sys),
            net: NetState::new(),
            disks: Vec::new(),
            processes: Vec::new(),
            #[cfg(target_os = "linux")]
            gpus: Vec::new(),
            sys,
            tick_rate_ms: 1000,
            selected_tab: 0,
            process_scroll: 0,
            disk_scroll: 0,
            #[cfg(target_os = "linux")]
            gpu_scroll: 0,
            #[cfg(all(target_os = "linux", feature = "nvidia"))]
            nvml: Nvml::init().ok(),
            t: get_translations(lang_code),
        }
    }

    /// Refreshes all metric sub-states from live system data.
    ///
    /// Called by the event loop once per [`tick_rate_ms`][App::tick_rate_ms].
    pub fn on_tick(&mut self) {
        self.sys.refresh_all();
        cpu::refresh(&mut self.cpu, &self.sys);
        memory::refresh(&mut self.mem, &self.sys);
        network::refresh(&mut self.net);
        disks::refresh(&mut self.disks);
        processes::refresh(&mut self.processes, &self.sys);
        #[cfg(target_os = "linux")]
        self.refresh_gpus();
    }

    /// Refreshes GPU data.
    ///
    /// Tries NVML first (when the `nvidia` feature is enabled); falls back to
    /// the AMD sysfs driver if NVML finds no devices.
    #[cfg(target_os = "linux")]
    fn refresh_gpus(&mut self) {
        #[cfg(feature = "nvidia")]
        {
            if gpu_metrics::nvidia::refresh(&mut self.gpus, &mut self.nvml) {
                return;
            }
        }
        gpu_metrics::amd::refresh(&mut self.gpus);
    }
}
