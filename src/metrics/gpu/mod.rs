pub mod amd;

#[cfg(feature = "nvidia")]
pub mod nvidia;

use super::HISTORY_LEN;

#[derive(Debug)]
pub struct GpuEntry {
    pub name: String,
    pub utilization: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub temperature: Option<u32>,
    pub power_watts: Option<f32>,
    pub util_history: Vec<f32>,
    pub mem_history: Vec<f32>,
}

impl GpuEntry {
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
