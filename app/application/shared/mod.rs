pub mod bgworker;
pub mod cache_provider;
pub mod command_handler;
pub mod dto;
pub mod event;
pub mod event_subscriber;
pub mod paging_query;
pub mod paging_result;
pub mod query_handler;
pub mod scheduler_job;

pub mod bgworker_impl {
    include!(concat!(env!("OUT_DIR"), "/bgworker.rs"));
}

pub mod event_subscriber_impl {
    include!(concat!(env!("OUT_DIR"), "/event_subscriber.rs"));
}

pub mod scheduler_job_impl {

    #[derive(Debug, Clone)]
    pub struct SchedulerJob {
        pub key: &'static str,
        pub name: &'static str,
        pub expr: &'static str,
    }
    include!(concat!(env!("OUT_DIR"), "/scheduler_job.rs"));
}
