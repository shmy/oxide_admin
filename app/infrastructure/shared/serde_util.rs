use serde::{Serialize, de::DeserializeOwned};

pub fn json_encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_default()
}
pub fn json_encode_to_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_encode() {
        let value = json_encode(&serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "city": "New York"
        }));

        assert!(value.len() > 0);
    }

    #[test]
    fn test_json_decode() {
        let value = json_decode::<serde_json::Value>(
            r#"{"name":"John Doe","age":30,"city":"New York"}"#.as_bytes(),
        );

        assert_eq!(
            value,
            serde_json::json!({
                "name": "John Doe",
                "age": 30,
                "city": "New York"
            })
        );
    }

    #[test]
    fn test_json_encode_to_value() {
        let value = json_encode_to_value(&serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "city": "New York"
        }));

        assert_eq!(
            value,
            serde_json::json!({
                "name": "John Doe",
                "age": 30,
                "city": "New York"
            })
        );
    }

    #[test]
    fn test_json_decode_from_value() {
        let value = json_decode_from_value::<serde_json::Value>(serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "city": "New York"
        }));

        assert_eq!(
            value,
            serde_json::json!({
                "name": "John Doe",
                "age": 30,
                "city": "New York"
            })
        );
    }
}
