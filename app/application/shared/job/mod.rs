use background_job::JobStorage;

use crate::shared::job::{
    delete_expired_kv_job::DeleteExpiredKvJob, delete_outdate_log_dir_job::DeleteOutdateLogDirJob,
    delete_outdate_temp_dir_job::DeleteOutdateTempDirJob,
    delete_unused_file_job::DeleteUnusedFileJob,
};

pub mod delete_expired_kv_job;
pub mod delete_outdate_log_dir_job;
pub mod delete_outdate_temp_dir_job;
pub mod delete_unused_file_job;

#[derive(Clone, Debug)]
pub struct Jobs {
    pub delete_expired_kv_job: JobStorage<DeleteExpiredKvJob>,
    pub delete_outdate_temp_dir_job: JobStorage<DeleteOutdateTempDirJob>,
    pub delete_outdate_log_dir_job: JobStorage<DeleteOutdateLogDirJob>,
    pub delete_unused_file_job: JobStorage<DeleteUnusedFileJob>,
}
