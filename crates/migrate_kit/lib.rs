pub mod error;
mod migrator;
pub use migrate_kit_macros::embed_dir;
pub use migrator::{Migration, Migrator};
