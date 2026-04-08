use sysinfo::System;

#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

pub const HISTORY_LEN: usize = 60;

#[derive(Debug)]
pub struct CpuState {
    pub usages: Vec<f32>,        // per-core current usage
    pub history: Vec<Vec<f32>>,  // per-core rolling history
    pub global_history: Vec<f32>,
}

#[derive(Debug)]
pub struct MemState {
    pub total: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub history: Vec<f32>, // percent used
}

#[derive(Debug)]
pub struct NetState {
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub rx_history: Vec<u64>,
    pub tx_history: Vec<u64>,
    prev_rx: u64,
    prev_tx: u64,
}

#[derive(Debug)]
pub struct DiskState {
    pub name: String,
    pub total: u64,
    pub used: u64,
    pub mount: String,
}

#[derive(Debug)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_kb: u64,
}

#[derive(Debug)]
pub struct GpuEntry {
    pub name: String,
    pub utilization: f32,   // percent
    pub mem_used: u64,      // bytes
    pub mem_total: u64,     // bytes
    pub temperature: Option<u32>, // celsius
    pub power_watts: Option<f32>,
    pub util_history: Vec<f32>,
    pub mem_history: Vec<f32>,
}

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

        let cpu = CpuState {
            usages: vec![0.0; cpu_count],
            history: vec![vec![0.0; HISTORY_LEN]; cpu_count],
            global_history: vec![0.0; HISTORY_LEN],
        };

        let mem = MemState {
            total: sys.total_memory(),
            used: sys.used_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            history: vec![0.0; HISTORY_LEN],
        };

        let net = NetState {
            rx_bytes_sec: 0,
            tx_bytes_sec: 0,
            rx_history: vec![0; HISTORY_LEN],
            tx_history: vec![0; HISTORY_LEN],
            prev_rx: 0,
            prev_tx: 0,
        };

        let disks = Vec::new();

        #[cfg(feature = "nvidia")]
        let nvml = Nvml::init().ok();

        App {
            sys,
            cpu,
            mem,
            net,
            disks,
            processes: Vec::new(),
            gpus: Vec::new(),
            tick_rate_ms: 1000,
            selected_tab: 0,
            process_scroll: 0,
            disk_scroll: 0,
            gpu_scroll: 0,
            #[cfg(feature = "nvidia")]
            nvml,
        }
    }

    pub fn on_tick(&mut self) {
        self.sys.refresh_all();
        self.refresh_cpu();
        self.refresh_mem();
        self.refresh_net();
        self.refresh_disks();
        self.refresh_processes();
        self.refresh_gpus();
    }

    fn refresh_cpu(&mut self) {
        let cpus = self.sys.cpus();
        let mut global_sum = 0.0_f32;

        for (i, cpu) in cpus.iter().enumerate() {
            let usage = cpu.cpu_usage();
            global_sum += usage;
            if i < self.cpu.usages.len() {
                self.cpu.usages[i] = usage;
                push_history(&mut self.cpu.history[i], usage);
            }
        }

        let global = if cpus.is_empty() {
            0.0
        } else {
            global_sum / cpus.len() as f32
        };
        push_history(&mut self.cpu.global_history, global);
    }

    fn refresh_mem(&mut self) {
        self.mem.total = self.sys.total_memory();
        self.mem.used = self.sys.used_memory();
        self.mem.swap_total = self.sys.total_swap();
        self.mem.swap_used = self.sys.used_swap();

        let pct = if self.mem.total > 0 {
            self.mem.used as f32 / self.mem.total as f32 * 100.0
        } else {
            0.0
        };
        push_history(&mut self.mem.history, pct);
    }

    fn refresh_net(&mut self) {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;
        for (_, data) in &networks {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }

        let rx_delta = total_rx.saturating_sub(self.net.prev_rx);
        let tx_delta = total_tx.saturating_sub(self.net.prev_tx);

        self.net.prev_rx = total_rx;
        self.net.prev_tx = total_tx;
        self.net.rx_bytes_sec = rx_delta;
        self.net.tx_bytes_sec = tx_delta;

        push_history(&mut self.net.rx_history, rx_delta);
        push_history(&mut self.net.tx_history, tx_delta);
    }

    fn refresh_disks(&mut self) {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        self.disks = disks
            .iter()
            .map(|d| DiskState {
                name: d.name().to_string_lossy().into_owned(),
                total: d.total_space(),
                used: d.total_space().saturating_sub(d.available_space()),
                mount: d.mount_point().to_string_lossy().into_owned(),
            })
            .collect();
    }

    fn refresh_processes(&mut self) {
        let mut procs: Vec<ProcessEntry> = self
            .sys
            .processes()
            .values()
            .map(|p| ProcessEntry {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage(),
                mem_kb: p.memory() / 1024,
            })
            .collect();

        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(100);
        self.processes = procs;
    }

    fn refresh_gpus(&mut self) {
        #[cfg(feature = "nvidia")]
        {
            self.refresh_gpus_nvml();
            if !self.gpus.is_empty() {
                return;
            }
        }
        self.refresh_gpus_amd();
    }

    #[cfg(feature = "nvidia")]
    fn refresh_gpus_nvml(&mut self) {
        let nvml = match self.nvml.take() {
            Some(n) => n,
            None => return,
        };
        let count = nvml.device_count().unwrap_or(0) as usize;
        if count == 0 {
            self.nvml = Some(nvml);
            return;
        }
        if self.gpus.len() != count {
            self.gpus = (0..count)
                .map(|i| {
                    let name = nvml
                        .device_by_index(i as u32)
                        .and_then(|d| d.name())
                        .unwrap_or_else(|_| format!("NVIDIA GPU {i}"));
                    GpuEntry {
                        name,
                        utilization: 0.0,
                        mem_used: 0,
                        mem_total: 0,
                        temperature: None,
                        power_watts: None,
                        util_history: vec![0.0; HISTORY_LEN],
                        mem_history: vec![0.0; HISTORY_LEN],
                    }
                })
                .collect();
        }
        for i in 0..count {
            if let Ok(device) = nvml.device_by_index(i as u32) {
                let util = device
                    .utilization_rates()
                    .map(|u| u.gpu as f32)
                    .unwrap_or(0.0);
                let (mem_used, mem_total) = device
                    .memory_info()
                    .map(|m| (m.used, m.total))
                    .unwrap_or((0, 0));
                let temperature = device
                    .temperature(
                        nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu,
                    )
                    .ok();
                let power_watts = device.power_usage().ok().map(|mw| mw as f32 / 1000.0);
                let gpu = &mut self.gpus[i];
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
        self.nvml = Some(nvml);
    }

    fn refresh_gpus_amd(&mut self) {
        use std::fs;
        use std::path::Path;

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
        if self.gpus.len() != card_paths.len() {
            self.gpus = card_paths
                .iter()
                .map(|p| GpuEntry {
                    name: amd_gpu_name(p),
                    utilization: 0.0,
                    mem_used: 0,
                    mem_total: 0,
                    temperature: None,
                    power_watts: None,
                    util_history: vec![0.0; HISTORY_LEN],
                    mem_history: vec![0.0; HISTORY_LEN],
                })
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
            let temperature = amd_hwmon_temp(&dev);
            let power_watts = amd_hwmon_power(&dev);
            let gpu = &mut self.gpus[i];
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
}

fn push_history<T: Copy>(history: &mut Vec<T>, value: T) {
    if history.len() >= HISTORY_LEN {
        history.remove(0);
    }
    history.push(value);
}

fn amd_gpu_name(card_path: &std::path::Path) -> String {
    use std::fs;
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

fn amd_hwmon_temp(dev_path: &std::path::Path) -> Option<u32> {
    use std::fs;
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

fn amd_hwmon_power(dev_path: &std::path::Path) -> Option<f32> {
    use std::fs;
    let hwmon = fs::read_dir(dev_path.join("hwmon")).ok()?;
    for entry in hwmon.filter_map(|e| e.ok()) {
        // power1_average is in microwatts
        if let Ok(val) = fs::read_to_string(entry.path().join("power1_average")) {
            if let Ok(uw) = val.trim().parse::<u64>() {
                return Some(uw as f32 / 1_000_000.0);
            }
        }
    }
    None
}
