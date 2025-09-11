use serde::{Serialize, de::DeserializeOwned};

pub fn json_encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_default()
}

pub fn json_decode<T: DeserializeOwned + Default>(bytes: &[u8]) -> T {
    serde_json::from_slice(bytes).unwrap_or_default()
}

pub fn json_encode_to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_default()
}

pub fn json_decode_from_value<T: DeserializeOwned + Default>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap_or_default()
}
