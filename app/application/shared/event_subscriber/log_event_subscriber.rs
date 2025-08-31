use anyhow::Result;
use event_derive::EventSubscriber;
use infrastructure::shared::event_bus::EventSubscriber;
use nject::injectable;

use crate::shared::event::Event;

#[derive(Debug, Clone, EventSubscriber)]
#[injectable]
pub struct LogEventSubscriber;

impl EventSubscriber<Event> for LogEventSubscriber {
    async fn on_received(&self, event: Event) -> Result<()> {
        tracing::info!("{self:?}: {event:?}");
        Ok(())
    }
}
