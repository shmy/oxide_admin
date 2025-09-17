use serde::Serialize;
use utoipa::ToSchema;

use crate::system::dto::{process_info::ProcessInfo, system_info::SystemInfo};

#[derive(Default, Debug, Serialize, ToSchema)]
pub struct SystemSnapshot {
    pub system: SystemInfo,
    pub process: Option<ProcessInfo>,
}
