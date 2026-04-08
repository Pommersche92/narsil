use sysinfo::System;

use super::{push_history, HISTORY_LEN};

#[derive(Debug)]
pub struct CpuState {
    pub usages: Vec<f32>,
    pub history: Vec<Vec<f32>>,
    pub global_history: Vec<f32>,
}

impl CpuState {
    pub fn new(cpu_count: usize) -> Self {
        Self {
            usages: vec![0.0; cpu_count],
            history: vec![vec![0.0; HISTORY_LEN]; cpu_count],
            global_history: vec![0.0; HISTORY_LEN],
        }
    }
}

pub fn refresh(state: &mut CpuState, sys: &System) {
    let cpus = sys.cpus();
    let mut global_sum = 0.0_f32;

    for (i, cpu) in cpus.iter().enumerate() {
        let usage = cpu.cpu_usage();
        global_sum += usage;
        if i < state.usages.len() {
            state.usages[i] = usage;
            push_history(&mut state.history[i], usage);
        }
    }

    let global = if cpus.is_empty() {
        0.0
    } else {
        global_sum / cpus.len() as f32
    };
    push_history(&mut state.global_history, global);
}
