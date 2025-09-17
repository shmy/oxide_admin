#[macro_export]
macro_rules! id {
    ($name: ident) => {
        use std::ops::Deref;

        use serde::{Deserialize, Serialize};

        use $crate::shared::id_generator::IdGenerator;

        #[derive(
            Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema, sqlx::Type,
        )]
        #[sqlx(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn generate() -> Self {
                Self(IdGenerator::primary_id())
            }

            pub fn new_unchecked(id: String) -> Self {
                Self(id)
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
