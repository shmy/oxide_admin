use bon::Builder;
use chrono::{DateTime, NaiveDateTime, Utc};
pub use chrono::{Datelike, Duration};

#[derive(Debug, Clone, Builder)]
pub struct ChronoTz {
    tz: chrono_tz::Tz,
}

impl ChronoTz {
    #[tracing::instrument]
    pub fn now(&self) -> NaiveDateTime {
        self.now_utc().with_timezone(&self.tz).naive_local()
    }

    #[tracing::instrument]
    pub fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
