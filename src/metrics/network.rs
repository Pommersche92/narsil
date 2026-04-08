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

//! Network throughput metrics.
//!
//! Computes per-tick RX/TX byte deltas across all network interfaces and
//! maintains a 60-sample rolling history.

use super::{push_history, HISTORY_LEN};

/// Throughput snapshot for the aggregate of all network interfaces.
#[derive(Debug)]
pub struct NetState {
    /// Bytes received per tick (approximates bytes/second at the default 1 Hz rate).
    pub rx_bytes_sec: u64,
    /// Bytes transmitted per tick.
    pub tx_bytes_sec: u64,
    /// 60-sample rolling history of received bytes per tick.
    pub rx_history: Vec<u64>,
    /// 60-sample rolling history of transmitted bytes per tick.
    pub tx_history: Vec<u64>,
    /// Cumulative RX byte counter from the previous tick, used to compute the delta.
    prev_rx: u64,
    /// Cumulative TX byte counter from the previous tick.
    prev_tx: u64,
}

impl NetState {
    /// Creates a zeroed [`NetState`].
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

/// Re-reads all network interface byte counters, computes per-tick RX/TX
/// deltas, and appends them to the rolling history buffers.
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
