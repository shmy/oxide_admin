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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::iam::event::IamEvent;

    use crate::shared::event::Event;

    #[tokio::test]
    async fn test_on_received() {
        let subscriber = LogEventSubscriber;
        let event = Event::Iam(IamEvent::UsersCreated { items: vec![] });
        assert!(subscriber.on_received(event).await.is_ok());
    }
}
