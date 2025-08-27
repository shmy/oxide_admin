use chrono::{DateTime, NaiveDateTime, Utc};
pub use chrono::{Datelike, Duration};
use nject::injectable;

use crate::shared::config::Config;

#[derive(Clone)]
#[injectable]
pub struct ChronoTz {
    config: Config,
}

impl ChronoTz {
    pub fn now(&self) -> NaiveDateTime {
        self.now_utc()
            .with_timezone(&self.config.database.timezone)
            .naive_local()
    }

    pub fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
