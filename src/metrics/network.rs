use super::{push_history, HISTORY_LEN};

#[derive(Debug)]
pub struct NetState {
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub rx_history: Vec<u64>,
    pub tx_history: Vec<u64>,
    prev_rx: u64,
    prev_tx: u64,
}

impl NetState {
    pub fn new() -> Self {
        Self {
            rx_bytes_sec: 0,
            tx_bytes_sec: 0,
            rx_history: vec![0; HISTORY_LEN],
            tx_history: vec![0; HISTORY_LEN],
            prev_rx: 0,
            prev_tx: 0,
        }
    }
}

pub fn refresh(state: &mut NetState) {
    use sysinfo::Networks;

    let networks = Networks::new_with_refreshed_list();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;
    for (_, data) in &networks {
        total_rx += data.total_received();
        total_tx += data.total_transmitted();
    }

    let rx_delta = total_rx.saturating_sub(state.prev_rx);
    let tx_delta = total_tx.saturating_sub(state.prev_tx);

    state.prev_rx = total_rx;
    state.prev_tx = total_tx;
    state.rx_bytes_sec = rx_delta;
    state.tx_bytes_sec = tx_delta;

    push_history(&mut state.rx_history, rx_delta);
    push_history(&mut state.tx_history, tx_delta);
}
