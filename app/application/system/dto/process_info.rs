use bon::Builder;
use serde::Serialize;

#[derive(Debug, Serialize, Builder)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    start_time: u64,
    exe: Option<String>,
    cwd: Option<String>,
}
