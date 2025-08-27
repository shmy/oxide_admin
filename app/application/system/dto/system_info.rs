use serde::Serialize;

use crate::system::dto::cpu::Cpu;

#[derive(Default, Debug, Serialize)]
pub struct SystemInfo {
    pub os_name: Option<String>,
    pub long_os_version: Option<String>,
    pub host_name: Option<String>,
    pub physical_core_count: Option<usize>,
    pub cpus: Vec<Cpu>,
    pub cpu_arch: String,
    pub total_memory: u64,
    pub total_swap: u64,
    pub boot_time: u64,
}
