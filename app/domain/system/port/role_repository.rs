use crate::{
    shared::{event_util::UpdatedEvent, port::domain_repository::DomainRepository},
    system::value_object::role_id::RoleId,
};

pub trait RoleRepository: DomainRepository {
    fn toggle_enabled(
        &self,
        ids: &[RoleId],
        enabled: bool,
    ) -> impl Future<Output = Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error>>;
}
