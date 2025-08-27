use crate::{
    iam::value_object::role_id::RoleId,
    shared::{domain_repository::DomainRepository, event_util::UpdatedEvent},
};

pub trait RoleRepository: DomainRepository {
    fn toggle_enabled(
        &self,
        ids: &[RoleId],
        enabled: bool,
    ) -> impl Future<Output = Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error>>;
}
