use anyhow::Result;
use infrastructure::shared::event_bus::EventSubscriber;

use crate::shared::event::Event;

#[derive(Debug, Clone)]
pub struct LogEventSubscriber;

impl EventSubscriber<Event> for LogEventSubscriber {
    async fn on_received(&self, event: Event) -> Result<()> {
        tracing::info!("{self:?}: {event:?}");
        Ok(())
    }
}
