mod iam_event_subscriber;
mod log_event_subscriber;

include!(concat!(env!("OUT_DIR"), "/register_subscribers.rs"));
