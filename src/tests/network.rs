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

//! Tests for [`crate::metrics::network`].
//!
//! Covers: [`NetState::new`] zeroed initial state and history dimensions,
//! and the `refresh` function (smoke test, history-length invariant, and
//! verification that rate fields are populated without panicking).

use crate::metrics::{
    network::{refresh, NetState},
    HISTORY_LEN,
};

// ── NetState::new ─────────────────────────────────────────────────────────────

/// A freshly constructed `NetState` has all rate counters set to zero.
#[test]
fn test_net_state_new_rates_are_zero() {
    let state = NetState::new();
    assert_eq!(state.rx_bytes_sec, 0);
    assert_eq!(state.tx_bytes_sec, 0);
}

/// A freshly constructed `NetState` has RX history of length `HISTORY_LEN`.
#[test]
fn test_net_state_new_rx_history_length() {
    let state = NetState::new();
    assert_eq!(state.rx_history.len(), HISTORY_LEN);
}

/// A freshly constructed `NetState` has TX history of length `HISTORY_LEN`.
#[test]
fn test_net_state_new_tx_history_length() {
    let state = NetState::new();
    assert_eq!(state.tx_history.len(), HISTORY_LEN);
}

/// All initial history samples are zero.
#[test]
fn test_net_state_new_histories_all_zero() {
    let state = NetState::new();
    assert!(state.rx_history.iter().all(|&v| v == 0));
    assert!(state.tx_history.iter().all(|&v| v == 0));
}

// ── refresh ───────────────────────────────────────────────────────────────────

/// `refresh` must not panic when called on a zeroed `NetState`.
#[test]
fn test_net_refresh_does_not_panic() {
    let mut state = NetState::new();
    refresh(&mut state);
}

/// After `refresh`, the RX and TX history lengths stay at `HISTORY_LEN`.
#[test]
fn test_net_refresh_histories_stay_capped() {
    let mut state = NetState::new();
    refresh(&mut state);
    assert_eq!(state.rx_history.len(), HISTORY_LEN);
    assert_eq!(state.tx_history.len(), HISTORY_LEN);
}

/// After two consecutive `refresh` calls, the most recent RX/TX sample in
/// the history matches the current `rx_bytes_sec` / `tx_bytes_sec` value
/// (the second call computes the true per-tick delta).
#[test]
fn test_net_refresh_last_history_matches_rate() {
    let mut state = NetState::new();
    refresh(&mut state); // primes prev_rx / prev_tx
    refresh(&mut state); // computes a real delta

    assert_eq!(
        *state.rx_history.last().unwrap(),
        state.rx_bytes_sec,
        "last RX history sample should equal rx_bytes_sec"
    );
    assert_eq!(
        *state.tx_history.last().unwrap(),
        state.tx_bytes_sec,
        "last TX history sample should equal tx_bytes_sec"
    );
}
