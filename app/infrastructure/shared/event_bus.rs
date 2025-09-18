use anyhow::Result;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::error;

pub trait EventSubscriber<T>: Clone + Send + Sync + 'static {
    fn on_received(&self, event: T) -> impl Future<Output = Result<()>> + Send;
}

pub struct EventBus<T: Clone + Send + Sync + 'static> {
    sender: Sender<T>,
}

impl<T: Clone + Send + Sync + 'static> EventBus<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn publish(&self, event: T) {
        if let Err(err) = self.sender.send(event) {
            tracing::error!(%err, "Failed to publish event");
        }
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: EventSubscriber<T>,
    {
        let receiver = self.sender.subscribe();
        tokio::spawn(Self::start_listening::<H>(receiver, handler));
    }

    async fn start_listening<H>(mut rx: Receiver<T>, handler: H)
    where
        H: EventSubscriber<T>,
    {
        while let Ok(event) = rx.recv().await {
            let h = handler.clone();
            tokio::spawn(async move {
                if let Err(e) = h.on_received(event).await {
                    error!("Event handler error: {e:?}");
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestEvent {
        value: i32,
    }

    impl TestEvent {
        pub fn new(value: i32) -> Self {
            Self { value }
        }
    }

    #[derive(Clone)]
    struct TestEventHandler {
        value: i32,
    }

    impl EventSubscriber<TestEvent> for TestEventHandler {
        fn on_received(&self, event: TestEvent) -> impl Future<Output = Result<()>> + Send {
            async move {
                assert_eq!(event.value, self.value);
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_publish() {
        let bus = EventBus::<TestEvent>::new(16);
        bus.publish(TestEvent::new(1));
        bus.publish(TestEvent::new(2));
        bus.publish(TestEvent::new(3));
    }

    #[tokio::test]
    async fn test_subscribe() {
        let bus = EventBus::<TestEvent>::new(16);
        let handler = TestEventHandler { value: 1 };
        bus.subscribe(handler.clone());
        bus.publish(TestEvent::new(1));
        bus.publish(TestEvent::new(2));
        bus.publish(TestEvent::new(3));
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
