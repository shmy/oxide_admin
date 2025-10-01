use domain::system::event::SystemEvent;
use infrastructure::shared::event_bus::EventBus;
use std::sync::LazyLock;
pub static EVENT_BUS: LazyLock<EventBus<Event>> = LazyLock::new(|| EventBus::new(64));
#[derive(Debug, Clone)]
pub enum Event {
    System(SystemEvent),
}
impl From<SystemEvent> for Event {
    fn from(value: SystemEvent) -> Self {
        Self::System(value)
    }
}
