use crate::EventSubscriber;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::error;

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
        H: EventSubscriber<T> + Send + Sync + 'static,
    {
        let h = std::sync::Arc::new(handler);
        let rx = self.sender.subscribe();
        tokio::spawn(Self::listen(h, rx));
    }

    async fn listen<H>(handler: std::sync::Arc<H>, mut rx: Receiver<T>)
    where
        H: EventSubscriber<T> + Send + Sync + 'static,
    {
        while let Ok(event) = rx.recv().await {
            let cloned_handler = handler.clone();
            tokio::spawn(async move {
                if let Err(err) = cloned_handler.on_received(event).await {
                    error!(error = %err, "Event handler error: {}", err);
                }
            });
        }
    }
}
