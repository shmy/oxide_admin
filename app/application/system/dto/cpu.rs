use bon::Builder;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Builder, ToSchema)]
pub struct Cpu {
    name: String,
    brand: String,
    vendor_id: String,
    frequency: u64,
}
