use serde::Deserialize;

use crate::response::common::{DeviceFlag, OnlineFlag};

#[derive(Debug, Deserialize)]
pub struct OnlineStatus {
    pub uid: String,
    pub online: OnlineFlag,
    pub device_flag: DeviceFlag,
}
