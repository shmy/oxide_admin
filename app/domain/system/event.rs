use crate::system::entity::access_log::AccessLog;
use crate::system::entity::sched::Sched;
#[derive(Debug, Clone)]
pub enum SystemEvent {
    SchedsDeleted { items: Vec<Sched> },
    AccessLogsCreated { items: Vec<AccessLog> },
}
