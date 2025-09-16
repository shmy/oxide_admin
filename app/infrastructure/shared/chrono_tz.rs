use chrono::{DateTime, NaiveDateTime, Utc};
pub use chrono::{Datelike, Duration};
use nject::injectable;

use crate::shared::config::Config;

#[derive(Debug, Clone)]
#[injectable]
pub struct ChronoTz {
    config: Config,
}

impl ChronoTz {
    #[tracing::instrument]
    pub fn now(&self) -> NaiveDateTime {
        self.now_utc()
            .with_timezone(&self.config.timezone)
            .naive_local()
    }

    #[tracing::instrument]
    pub fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
