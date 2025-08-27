pub trait DomainRepository {
    type Entity;
    type EntityId;
    type Error;
    fn by_id(&self, id: &Self::EntityId)
    -> impl Future<Output = Result<Self::Entity, Self::Error>>;

    fn save(&self, entity: Self::Entity)
    -> impl Future<Output = Result<Self::Entity, Self::Error>>;

    fn batch_delete(
        &self,
        ids: &[Self::EntityId],
    ) -> impl Future<Output = Result<Vec<Self::Entity>, Self::Error>>;
}
