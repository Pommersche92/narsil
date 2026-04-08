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

//! Disk usage metrics.
//!
//! Refreshes the list of mounted filesystems and their total/used space via
//! [`sysinfo::Disks`].

/// Usage snapshot for a single mounted filesystem.
#[derive(Debug)]
pub struct DiskState {
    /// Device or filesystem label (e.g. `"sda1"`).
    pub name: String,
    /// Total capacity in bytes.
    pub total: u64,
    /// Used capacity in bytes (= total − available).
    pub used: u64,
    /// Mount point path (e.g. `"/"`).
    pub mount: String,
}

/// Replaces `disks` with a freshly queried list of mounted filesystems and
/// their current usage figures.
pub fn refresh(disks: &mut Vec<DiskState>) {
    use sysinfo::Disks;

    let sysinfo_disks = Disks::new_with_refreshed_list();
    *disks = sysinfo_disks
        .iter()
        .map(|d| DiskState {
            name: d.name().to_string_lossy().into_owned(),
            total: d.total_space(),
            used: d.total_space().saturating_sub(d.available_space()),
            mount: d.mount_point().to_string_lossy().into_owned(),
        })
        .collect();
}
