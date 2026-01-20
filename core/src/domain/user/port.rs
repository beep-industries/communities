use std::sync::Arc;

use futures_util::lock::Mutex;

use crate::{domain::user::entities::User, infrastructure::user::repositories::error::UserError};

pub trait UserRepository: Send + Sync {
    fn get_user_by_username(
        &self,
        username: &String,
    ) -> impl Future<Output = Result<Option<User>, UserError>> + Send;
}

#[derive(Clone)]
pub struct MockUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl UserRepository for MockUserRepository {
    async fn get_user_by_username(&self, username: &String) -> Result<Option<User>, UserError> {
        Ok(None)
    }
}
