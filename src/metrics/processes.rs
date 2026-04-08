use sysinfo::System;

#[derive(Debug)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_kb: u64,
}

pub fn refresh(processes: &mut Vec<ProcessEntry>, sys: &System) {
    let mut procs: Vec<ProcessEntry> = sys
        .processes()
        .values()
        .map(|p| ProcessEntry {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu: p.cpu_usage(),
            mem_kb: p.memory() / 1024,
        })
        .collect();

    procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(100);
    *processes = procs;
}
