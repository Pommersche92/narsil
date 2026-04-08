use sysinfo::System;

#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

use crate::metrics::{
    cpu, disks, memory, network, processes,
    gpu as gpu_metrics,
    CpuState, DiskState, GpuEntry, MemState, NetState, ProcessEntry,
};

pub struct App {
    pub sys: System,
    pub cpu: CpuState,
    pub mem: MemState,
    pub net: NetState,
    pub disks: Vec<DiskState>,
    pub processes: Vec<ProcessEntry>,
    pub gpus: Vec<GpuEntry>,
    pub tick_rate_ms: u64,
    pub selected_tab: usize,
    pub process_scroll: usize,
    pub disk_scroll: usize,
    pub gpu_scroll: usize,
    #[cfg(feature = "nvidia")]
    pub(crate) nvml: Option<Nvml>,
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_count = sys.cpus().len().max(1);

        App {
            cpu: CpuState::new(cpu_count),
            mem: MemState::new(&sys),
            net: NetState::new(),
            disks: Vec::new(),
            processes: Vec::new(),
            gpus: Vec::new(),
            sys,
            tick_rate_ms: 1000,
            selected_tab: 0,
            process_scroll: 0,
            disk_scroll: 0,
            gpu_scroll: 0,
            #[cfg(feature = "nvidia")]
            nvml: Nvml::init().ok(),
        }
    }

    pub fn on_tick(&mut self) {
        self.sys.refresh_all();
        cpu::refresh(&mut self.cpu, &self.sys);
        memory::refresh(&mut self.mem, &self.sys);
        network::refresh(&mut self.net);
        disks::refresh(&mut self.disks);
        processes::refresh(&mut self.processes, &self.sys);
        self.refresh_gpus();
    }

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
