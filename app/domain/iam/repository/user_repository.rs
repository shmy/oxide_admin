use crate::{
    iam::value_object::user_id::UserId,
    shared::{domain_repository::DomainRepository, event_util::UpdatedEvent},
};

pub trait UserRepository: DomainRepository {
    fn by_account(
        &self,
        account: String,
    ) -> impl Future<Output = Result<Self::Entity, Self::Error>>;
    fn by_refresh_token(
        &self,
        refresh_token: String,
    ) -> impl Future<Output = Result<Self::Entity, Self::Error>>;
    fn toggle_enabled(
        &self,
        ids: &[UserId],
        enabled: bool,
    ) -> impl Future<Output = Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error>>;
}
