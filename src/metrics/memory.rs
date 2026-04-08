use sysinfo::System;

use super::{push_history, HISTORY_LEN};

#[derive(Debug)]
pub struct MemState {
    pub total: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub history: Vec<f32>,
}

impl MemState {
    pub fn new(sys: &System) -> Self {
        Self {
            total: sys.total_memory(),
            used: sys.used_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            history: vec![0.0; HISTORY_LEN],
        }
    }
}

pub fn refresh(state: &mut MemState, sys: &System) {
    state.total = sys.total_memory();
    state.used = sys.used_memory();
    state.swap_total = sys.total_swap();
    state.swap_used = sys.used_swap();

    let pct = if state.total > 0 {
        state.used as f32 / state.total as f32 * 100.0
    } else {
        0.0
    };
    push_history(&mut state.history, pct);
}
