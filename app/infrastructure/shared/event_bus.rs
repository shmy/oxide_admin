use anyhow::Result;
use std::fmt::Debug;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{error, info};

pub trait EventSubscriber<T>: Clone + Debug + Send + Sync + 'static {
    fn on_received(&self, event: T) -> impl Future<Output = Result<()>> + Send;
}

pub struct EventBus<T: Debug + Clone + Send + Sync + 'static> {
    sender: Sender<T>,
}

impl<T: Debug + Clone + Send + Sync + 'static> EventBus<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn publish(&self, event: T) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: EventSubscriber<T>,
    {
        let receiver = self.sender.subscribe();
        info!("Event subscriber [{:?}] has been registered", &handler);
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
