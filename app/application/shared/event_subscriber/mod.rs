pub mod iam_event_subscriber;
pub mod log_event_subscriber;

include!(concat!(env!("OUT_DIR"), "/register_subscribers.rs"));
