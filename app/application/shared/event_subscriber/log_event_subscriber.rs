use crate::shared::event::Event;
use infrastructure::error::InfrastructureResult;
use infrastructure::shared::event_bus::EventSubscriber;
use nject::injectable;

#[derive(Clone)]
#[injectable]
pub struct LogEventSubscriber;

impl EventSubscriber<Event> for LogEventSubscriber {
    async fn on_received(&self, event: Event) -> InfrastructureResult<()> {
        tracing::info!("on_received: {event:?}");
        Ok(())
    }
}
