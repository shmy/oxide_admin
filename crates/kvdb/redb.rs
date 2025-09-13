use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info};

use redb::{Database, ReadableDatabase as _, ReadableTable as _, TableDefinition};

use crate::{KvdbTrait, serde_util};

pub struct RedbKvdb {
    db: Arc<Database>,
    sched: Arc<Mutex<JobScheduler>>,
}

impl Debug for RedbKvdb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbKvdb").finish()
    }
}

impl RedbKvdb {
    pub async fn try_new(path: impl AsRef<Path>) -> Result<Self> {
        let db_path = path.as_ref();
        let db = Database::create(db_path)?;
        info!("Redb {} connected", db_path.display());
        let sched = JobScheduler::new().await?;

        let instance = Self {
            db: Arc::new(db),
            sched: Arc::new(Mutex::new(sched)),
        };
        let cloned_instance = instance.clone();
        tokio::spawn(async move {
            if let Err(err) = cloned_instance.start_scheduler().await {
                error!(%err, "Start scheduler failed");
            }
        });
        Ok(instance)
    }
}

impl Clone for RedbKvdb {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            sched: self.sched.clone(),
        }
    }
}

impl RedbKvdb {
    async fn start_scheduler(&self) -> Result<()> {
        let guard = self.sched.lock().await;
        let self_clone = self.clone();
        // every hour
        guard
            .add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
                let self_inner = self_clone.clone();
                Box::pin(async move {
                    if let Err(err) = self_inner.delete_expired().await {
                        error!(%err, "Delete expired failed");
                    }
                })
            })?)
            .await?;
        guard.start().await?;
        Ok(())
    }

    async fn delete_expired(&self) -> Result<()> {
        debug!("Start delete_expired");
        let tx = self.db.begin_read()?;
        let table = tx.open_table(TABLE)?;
        let iter = table.iter()?;
        let keys = iter
            .filter_map(|access| {
                if let Ok((key, value)) = access {
                    let key = key.value().to_string();
                    let value = value.value().to_vec();
                    let s = serde_util::rmp_decode::<KvValue>(&value).ok()?;
                    if let Some(expires_at) = s.expires_at {
                        let now = Utc::now().timestamp();
                        debug!(
                            "Found key: {}, now: {}, expires_at: {}",
                            key, now, expires_at
                        );
                        if now > expires_at {
                            return Some(key);
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        drop(tx);
        if !keys.is_empty() {
            let tx = self.db.begin_write()?;
            {
                let mut table = tx.open_table(TABLE)?;
                for key in keys {
                    info!("Delete expired key: {}", key);
                    let _ = table.remove(key.as_str());
                }
            }
            tx.commit()?;
        }
        Ok(())
    }
}

const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("app_data");

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KvValue {
    pub value: Vec<u8>,
    pub expires_at: Option<i64>,
}

impl KvdbTrait for RedbKvdb {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let tx = self.db.begin_read().ok()?;
        let table = tx.open_table(TABLE).ok()?;
        let value_opt = table.get(key).ok()?;
        if let Some(value) = value_opt {
            let kv: KvValue = serde_util::rmp_decode(value.value()).ok()?;
            if let Some(expires_at) = kv.expires_at {
                let now = Utc::now().timestamp();
                if now > expires_at {
                    return None;
                }
            }
            let val: T = serde_util::rmp_decode(&kv.value).ok()?;
            return Some(val);
        }
        None
    }

    async fn set_with_ex<T: Serialize>(
        &self,
        key: &str,
        value: T,
        duration: Duration,
    ) -> Result<()> {
        let tx = self.db.begin_write()?;
        let now = Utc::now();
        let expires_at = now + duration;
        let value = serde_util::rmp_encode(&KvValue {
            value: serde_util::rmp_encode(&value)?,
            expires_at: Some(expires_at.timestamp()),
        })?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.insert(key, value.as_slice())?;
        }
        tx.commit()?;
        Ok(())
    }

    async fn set_with_ex_at<T: Serialize>(
        &self,
        key: &str,
        value: T,
        expires_at: i64,
    ) -> Result<()> {
        let tx = self.db.begin_write()?;
        let value = serde_util::rmp_encode(&KvValue {
            value: serde_util::rmp_encode(&value)?,
            expires_at: Some(expires_at),
        })?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.insert(key, value.as_slice())?;
        }
        tx.commit()?;
        Ok(())
    }

    async fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let tx = self.db.begin_write()?;
        let value = serde_util::rmp_encode(&KvValue {
            value: serde_util::rmp_encode(&value)?,
            expires_at: None,
        })?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.insert(key, value.as_slice())?;
        }
        tx.commit()?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.remove(key)?;
        }
        tx.commit()?;
        Ok(())
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(TABLE)?;
        let iter = table.iter()?;
        let keys = iter
            .filter_map(|access| {
                if let Ok((key, _)) = access {
                    let key = key.value().to_string();
                    if key.starts_with(prefix) {
                        return Some(key);
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        drop(tx);
        if !keys.is_empty() {
            let tx = self.db.begin_write()?;
            {
                let mut table = tx.open_table(TABLE)?;
                for key in keys {
                    let _ = table.remove(key.as_str());
                }
            }
            tx.commit()?;
        }
        Ok(())
    }

    async fn close(&self) {
        let mut guard = self.sched.lock().await;
        if let Err(err) = guard.shutdown().await {
            error!(%err, "Failed to shutdown scheduler");
        }
    }
}
