use crate::domain::{
    common::CoreError,
    server::entities::{InsertServerInput, Server, ServerId},
};

pub trait ServerRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;
    fn find_by_id(
        &self,
        id: &ServerId,
    ) -> impl Future<Output = Result<Option<Server>, CoreError>> + Send;
}
