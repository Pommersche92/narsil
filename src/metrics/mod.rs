pub mod cpu;
pub mod disks;
pub mod memory;
pub mod network;
pub mod processes;
pub mod gpu;

pub use cpu::CpuState;
pub use disks::DiskState;
pub use memory::MemState;
pub use network::NetState;
pub use processes::ProcessEntry;
pub use gpu::GpuEntry;

pub const HISTORY_LEN: usize = 60;

pub fn push_history<T: Copy>(history: &mut Vec<T>, value: T) {
    if history.len() >= HISTORY_LEN {
        history.remove(0);
    }
    history.push(value);
}
