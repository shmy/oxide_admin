use anyhow::Result;
use infrastructure::shared::event_bus::EventSubscriber;
use nject::injectable;

use crate::shared::event::Event;

#[derive(Clone)]
#[injectable]
pub struct LogEventSubscriber;

impl EventSubscriber<Event> for LogEventSubscriber {
    async fn on_received(&self, event: Event) -> Result<()> {
        tracing::info!("on_received: {event:?}");
        Ok(())
    }
}
