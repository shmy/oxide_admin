use domain::{
    auth::event::AuthEvent, organization::event::OrganizationEvent, system::event::SystemEvent,
};
use event_kit::EventBus;
use std::sync::LazyLock;
pub static EVENT_BUS: LazyLock<EventBus<Event>> = LazyLock::new(|| EventBus::new(64));
#[derive(Debug, Clone)]
pub enum Event {
    Auth(AuthEvent),
    Organization(OrganizationEvent),
    System(SystemEvent),
}
impl From<AuthEvent> for Event {
    fn from(value: AuthEvent) -> Self {
        Self::Auth(value)
    }
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
