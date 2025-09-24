use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub fn rmp_encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(value)?)
}

pub fn rmp_decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    Ok(rmp_serde::from_slice(bytes)?)
}
