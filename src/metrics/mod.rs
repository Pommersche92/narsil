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

//! System-metric data modules.
//!
//! Each sub-module owns one strongly-typed state struct and a free
//! `refresh()` function that updates it. The constant [`HISTORY_LEN`] and
//! the helper [`push_history`] are shared across all sub-modules.

pub mod cpu;
pub mod disks;
pub mod memory;
pub mod network;
pub mod processes;
#[cfg(target_os = "linux")]
pub mod gpu;

pub use cpu::CpuState;
pub use disks::DiskState;
pub use memory::MemState;
pub use network::NetState;
pub use processes::ProcessEntry;
#[cfg(target_os = "linux")]
pub use gpu::GpuEntry;

/// Number of samples retained in each rolling history buffer.
///
/// At the default 1 Hz tick rate this corresponds to ~60 seconds of history.
pub const HISTORY_LEN: usize = 60;

/// Appends `value` to `history`, evicting the oldest sample when the buffer
/// has reached [`HISTORY_LEN`] entries.
pub fn push_history<T: Copy>(history: &mut Vec<T>, value: T) {
    if history.len() >= HISTORY_LEN {
        history.remove(0);
    }
    history.push(value);
}
