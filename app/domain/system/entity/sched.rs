use bon::Builder;

use crate::system::value_object::sched_id::SchedId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Sched {
    pub id: SchedId,
    pub key: String,
    pub name: String,
    pub expr: String,
    pub succeed: bool,
    pub result: String,
    pub run_at: chrono::NaiveDateTime,
    pub duration_ms: i64,
}
