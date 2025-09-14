use std::fmt::Debug;

pub trait QueryHandler: Debug {
    type Query;
    type Output;
    type Error;
    fn query(&self, query: Self::Query) -> impl Future<Output = Result<Self::Output, Self::Error>>;
}
