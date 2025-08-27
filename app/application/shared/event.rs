use std::sync::LazyLock;

use domain::iam::event::IamEvent;
use infrastructure::shared::event_bus::EventBus;

pub static EVENT_BUS: LazyLock<EventBus<Event>> = LazyLock::new(|| EventBus::new(64));

#[derive(Debug, Clone)]
pub enum Event {
    Iam(IamEvent),
}

impl From<IamEvent> for Event {
    fn from(event: IamEvent) -> Self {
        Self::Iam(event)
    }
}
