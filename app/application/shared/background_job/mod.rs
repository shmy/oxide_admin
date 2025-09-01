pub mod delete_expired_kv_job;
pub mod delete_outdate_log_dir_job;
pub mod delete_outdate_temp_dir_job;
pub mod delete_unused_file_job;

include!(concat!(env!("OUT_DIR"), "/jobs.rs"));
