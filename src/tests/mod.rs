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

//! Unit test suite for **narsil**.
//!
//! Tests are split into one sub-module per concern. The entire module tree
//! is compiled only in test builds (`#[cfg(test)]`).
//!
//! | Module | Covers |
//! |---|---|
//! | [`push_history`] | `metrics::push_history` helper |
//! | [`helpers`] | `ui::helpers` formatting and colour utilities |
//! | [`cpu`] | `metrics::cpu` state and refresh |
//! | [`memory`] | `metrics::memory` state and refresh |
//! | [`network`] | `metrics::network` state and refresh |
//! | [`disks`] | `metrics::disks` state and refresh |
//! | [`processes`] | `metrics::processes` state and refresh |
//! | [`gpu`] | `metrics::gpu` state and AMD refresh |
//! | [`split_gauge`] | `ui::widgets::SplitGauge` rendering |

mod push_history;
mod helpers;
mod cpu;
mod memory;
mod network;
mod disks;
mod processes;
#[cfg(target_os = "linux")]
mod gpu;
mod split_gauge;
