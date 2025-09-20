use anyhow::Result;
use nject::injectable;
use std::sync::LazyLock;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::system::dto::{
    cpu::Cpu, process_info::ProcessInfo, system_info::SystemInfo, system_snapshot::SystemSnapshot,
};

static SNAPSHOT: LazyLock<SystemSnapshot> = LazyLock::new(SystemService::build_snapshot);

#[derive(Debug)]
#[injectable]
pub struct SystemService;

impl SystemService {
    #[tracing::instrument]
    pub async fn info(&self) -> Result<&'static SystemSnapshot> {
        Ok(&SNAPSHOT)
    }

    #[tracing::instrument]
    fn build_snapshot() -> SystemSnapshot {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        if let Ok(pid) = sysinfo::get_current_pid() {
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing()
                    .with_memory()
                    .with_cpu()
                    .with_exe(UpdateKind::OnlyIfNotSet)
                    .with_cwd(UpdateKind::OnlyIfNotSet),
            );
        }
        let mut snapshot = SystemSnapshot {
            system: SystemInfo {
                os_name: System::name(),
                long_os_version: System::long_os_version(),
                host_name: System::host_name(),
                physical_core_count: System::physical_core_count(),
                cpus: sys
                    .cpus()
                    .iter()
                    .map(|cpu| {
                        Cpu::builder()
                            .name(cpu.name().to_string())
                            .brand(cpu.brand().to_string())
                            .frequency(cpu.frequency())
                            .vendor_id(cpu.vendor_id().to_string())
                            .build()
                    })
                    .collect(),
                cpu_arch: System::cpu_arch(),
                total_memory: sys.total_memory(),
                total_swap: sys.total_swap(),
                boot_time: System::boot_time(),
            },
            process: None,
        };
        if let Ok(pid) = sysinfo::get_current_pid()
            && let Some(process) = sys.process(pid)
        {
            let process = ProcessInfo::builder()
                .pid(pid.as_u32())
                .name(process.name().to_string_lossy().to_string())
                .start_time(process.start_time())
                .maybe_exe(process.exe().map(|x| x.to_string_lossy().to_string()))
                .maybe_cwd(process.cwd().map(|x| x.to_string_lossy().to_string()))
                .build();
            snapshot.process = Some(process)
        }

        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_info() {
        let service = SystemService;
        assert!(service.info().await.is_ok());
    }
}
