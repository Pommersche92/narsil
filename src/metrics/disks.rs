#[derive(Debug)]
pub struct DiskState {
    pub name: String,
    pub total: u64,
    pub used: u64,
    pub mount: String,
}

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
