use domain::{organization::event::OrganizationEvent, system::event::SystemEvent};
use infrastructure::shared::event_bus::EventBus;
use std::sync::LazyLock;
pub static EVENT_BUS: LazyLock<EventBus<Event>> = LazyLock::new(|| EventBus::new(64));
#[derive(Debug, Clone)]
pub enum Event {
    Organization(OrganizationEvent),
    System(SystemEvent),
}
impl From<OrganizationEvent> for Event {
    fn from(value: OrganizationEvent) -> Self {
        Self::Organization(value)
    }
}

impl From<SystemEvent> for Event {
    fn from(value: SystemEvent) -> Self {
        Self::System(value)
    }
}
