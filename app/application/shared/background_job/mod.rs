pub mod delete_expired_kv_cron_job;
pub mod delete_outdate_log_dir_cron_job;
pub mod delete_outdate_temp_dir_cron_job;
pub mod delete_unused_file_cron_job;

include!(concat!(env!("OUT_DIR"), "/jobs.rs"));
