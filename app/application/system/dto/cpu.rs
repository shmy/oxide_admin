use bon::Builder;
use serde::Serialize;

#[derive(Debug, Serialize, Builder)]
pub struct Cpu {
    name: String,
    brand: String,
    vendor_id: String,
    frequency: u64,
}
