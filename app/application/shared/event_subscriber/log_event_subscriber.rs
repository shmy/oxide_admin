use crate::shared::event::Event;
use event_kit::{EventSubscriber, error::Result};
use nject::injectable;

#[derive(Clone)]
#[injectable]
pub struct LogEventSubscriber;

impl EventSubscriber<Event> for LogEventSubscriber {
    async fn on_received(&self, event: Event) -> Result<()> {
        tracing::info!("on_received: {event:?}");
        Ok(())
    }
}
