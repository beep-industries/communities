use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::CoreError;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct UserId(pub Uuid);

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        UserId(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        UserId(
            Uuid::parse_str(&s)
                .map_err(|e| CoreError::UnknownError {
                    message: e.to_string(),
                })
                .unwrap(),
        )
    }
}

// Is used to map database rows to domain entities
#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRow {
    pub user_id_1: UserId,
    pub user_id_2: UserId,

    pub created_at: DateTime<Utc>,
}

// Is used in API responses
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Friend {
    pub user_id_1: UserId,
    pub user_id_2: UserId,

    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFriendInput {
    pub user_id_1: UserId,
    pub user_id_2: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFriendInput {
    pub user_id_1: UserId,
    pub user_id_2: UserId,
}

// Is used to map database rows to domain entities
#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequestRow {
    pub user_id_requested: UserId,
    pub user_id_invited: UserId,

    pub created_at: DateTime<Utc>,
}

// Is used in API responses
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FriendRequest {
    pub user_id_requested: UserId,
    pub user_id_invited: UserId,
    pub status: i16,

    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFriendRequestInput {
    pub user_pseudo_invited: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AcceptFriendRequestInput {
    pub user_id_requested: UserId,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeclineFriendRequestInput {
    pub user_id_requested: UserId,
}
