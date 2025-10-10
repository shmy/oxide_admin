use crate::{error::ApplicationResult, shared::cache_provider::CACHE_PREFIX};
use cache_kit::{Cache, CacheTrait as _};
use nject::injectable;
use serde::Serialize;
use std::sync::LazyLock;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use utoipa::ToSchema;

use crate::system::dto::{
    cpu::Cpu, process_info::ProcessInfo, system_info::SystemInfo, system_snapshot::SystemSnapshot,
};

static SNAPSHOT: LazyLock<SystemSnapshot> = LazyLock::new(SystemService::build_snapshot);

#[derive(Debug)]
#[injectable]
pub struct SystemService {
    cache: Cache,
}

impl SystemService {
    #[tracing::instrument]
    pub async fn info(&self) -> ApplicationResult<&'static SystemSnapshot> {
        Ok(&SNAPSHOT)
    }

    #[tracing::instrument]
    pub async fn cache_tree(&self) -> ApplicationResult<Vec<CacheTreeItem>> {
        let items = self.cache.iter_prefix(CACHE_PREFIX).await?;
        let mut roots: Vec<CacheTreeItem> = Vec::new();

        for item in items {
            let parts: Vec<&str> = item.key.split(':').collect();
            if parts.is_empty() {
                continue;
            }

            let root_label = parts[0];
            if let Some(root) = roots.iter_mut().find(|r| r.label == root_label) {
                let root_value = root.value.clone();
                root.insert(&parts[1..], &root_value);
            } else {
                let mut node = CacheTreeItem::new(root_label, None, parts.len() == 1);
                if parts.len() > 1 {
                    let parent_value = node.value.clone();
                    node.insert(&parts[1..], &parent_value);
                }
                roots.push(node);
            }
        }
        Ok(roots)
    }

    #[tracing::instrument]
    pub async fn retrieve_cache(&self, key: &str) -> Option<RetrieveCacheItem> {
        self.cache.get_raw_string(key).await.and_then(|item| {
            let item = RetrieveCacheItem {
                value: serde_json::to_string_pretty(&item.value).ok()?,
                expired_at: item.expired_at,
            };
            Some(item)
        })
    }

    #[tracing::instrument]
    pub async fn delete_cache_by_prefix(&self, prefix: &str) -> ApplicationResult<()> {
        self.cache.delete_prefix(prefix).await?;
        Ok(())
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

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct CacheTreeItem {
    label: String,
    value: String,
    #[schema(no_recursion)]
    children: Vec<CacheTreeItem>,
}

impl CacheTreeItem {
    fn new(label: &str, parent_value: Option<&str>, is_leaf: bool) -> Self {
        // ✅ 如果不是叶子，就在末尾加 `:`
        let value = if let Some(parent) = parent_value {
            if is_leaf {
                format!("{parent}{label}")
            } else {
                format!("{parent}{label}:")
            }
        } else if is_leaf {
            label.to_string()
        } else {
            format!("{label}:")
        };

        Self {
            label: label.to_string(),
            value,
            children: Vec::new(),
        }
    }

    fn insert(&mut self, parts: &[&str], parent_value: &str) {
        if parts.is_empty() {
            return;
        }

        let part = parts[0];
        let is_leaf = parts.len() == 1;

        if let Some(child) = self.children.iter_mut().find(|c| c.label == part) {
            let child_value = child.value.clone();
            child.insert(&parts[1..], &child_value);
        } else {
            let mut node = CacheTreeItem::new(part, Some(parent_value), is_leaf);
            if parts.len() > 1 {
                let next_parent_value = node.value.clone();
                node.insert(&parts[1..], &next_parent_value);
            }
            self.children.push(node);
        }
    }
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct RetrieveCacheItem {
    pub value: String,
    pub expired_at: Option<u64>,
}
