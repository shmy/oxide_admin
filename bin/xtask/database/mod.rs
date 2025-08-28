use anyhow::Result;
use serde::Serialize;
pub mod postgres;
// pub mod sqlite;
pub trait TableInfoTrait {
    fn table_info(&self, table: &str) -> impl Future<Output = Result<Vec<Field>>>;
}

#[derive(Debug, Serialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
}
