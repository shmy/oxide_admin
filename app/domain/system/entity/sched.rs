use bon::Builder;

use crate::system::value_object::sched_id::SchedId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Sched {
    pub id: SchedId,
    pub key: String,
    pub name: String,
    pub schedule: String,
    pub succeed: bool,
    pub output: String,
    pub run_at: chrono::NaiveDateTime,
    pub duration_ms: i64,
}

impl Sched {
    pub fn update_key(&mut self, key: String) {
        self.key = key;
    }
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn update_schedule(&mut self, schedule: String) {
        self.schedule = schedule;
    }
    pub fn update_succeed(&mut self, succeed: bool) {
        self.succeed = succeed;
    }
    pub fn update_output(&mut self, output: String) {
        self.output = output;
    }
    pub fn update_run_at(&mut self, run_at: chrono::NaiveDateTime) {
        self.run_at = run_at;
    }
    pub fn update_duration_ms(&mut self, duration_ms: i64) {
        self.duration_ms = duration_ms;
    }
}
