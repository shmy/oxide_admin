use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub status: i32,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OnlineFlag {
    Offline = 0,
    Online = 1,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DeviceFlag {
    App = 0,
    Web = 1,
    Desktop = 2,
}
