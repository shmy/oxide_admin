pub mod error;
mod tokio_channel;
use error::Result;
pub use tokio_channel::*;

pub trait EventSubscriber<T>: Clone + Send + Sync + 'static {
    fn on_received(&self, event: T) -> impl Future<Output = Result<()>> + Send;
}
