use serde::Serialize;

use crate::system::dto::{process_info::ProcessInfo, system_info::SystemInfo};

#[derive(Default, Debug, Serialize)]
pub struct SystemSnapshot {
    pub system: SystemInfo,
    pub process: Option<ProcessInfo>,
}
