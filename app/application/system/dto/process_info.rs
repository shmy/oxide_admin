use bon::Builder;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Builder, ToSchema)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    start_time: u64,
    exe: Option<String>,
    cwd: Option<String>,
}
