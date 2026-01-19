use chrono::Utc;
use communities_core::domain::friend::entities::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub exp: i64,  // expiration timestamp
    pub iat: i64,  // issued at timestamp
}

impl Claims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}
