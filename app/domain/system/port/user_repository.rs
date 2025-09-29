use crate::{
    system::value_object::user_id::UserId,
    shared::{event_util::UpdatedEvent, port::domain_repository::DomainRepository},
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
