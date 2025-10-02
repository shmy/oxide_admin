pub mod background_worker;
pub mod cache_provider;
pub mod command_handler;
pub mod dto;
pub mod event;
pub mod event_subscriber;
pub mod paging_query;
pub mod paging_result;
pub mod query_handler;
pub mod scheduler_job;

pub mod background_worker_impl {
    include!(concat!(env!("OUT_DIR"), "/background_worker.rs"));
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
