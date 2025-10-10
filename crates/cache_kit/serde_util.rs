use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub fn cbor_encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(minicbor_serde::to_vec(value)?)
}

pub fn cbor_decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    Ok(minicbor_serde::from_slice(bytes)?)
}
