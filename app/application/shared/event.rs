use std::sync::LazyLock;

use domain::system::event::IamEvent;
use infrastructure::shared::event_bus::EventBus;

pub static EVENT_BUS: LazyLock<EventBus<Event>> = LazyLock::new(|| EventBus::new(64));

#[derive(Debug, Clone)]
pub enum Event {
    Iam(IamEvent),
}

impl From<IamEvent> for Event {
    fn from(value: IamEvent) -> Self {
        Self::Iam(value)
    }
}
