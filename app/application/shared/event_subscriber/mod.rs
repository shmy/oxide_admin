use crate::shared::event::{EVENT_BUS, Event};
use infrastructure::shared::{event_bus::EventBus, provider::Provider};

pub mod iam_event_subscriber;
pub mod log_event_subscriber;

pub fn register_subscribers(provider: &Provider) {
    for item in inventory::iter::<EventRegistry> {
        (item.register)(&EVENT_BUS, provider);
    }
}

#[derive(Debug)]
pub struct EventRegistry {
    register: fn(&'static EventBus<Event>, &Provider),
}

impl EventRegistry {
    pub const fn new(register: fn(&'static EventBus<Event>, &Provider)) -> Self {
        Self { register }
    }
}
inventory::collect!(EventRegistry);
