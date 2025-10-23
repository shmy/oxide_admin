mod geocoder;

pub use geocoder::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TencentLocationResponse<T> {
    pub status: u8,
    pub result: T,
}
