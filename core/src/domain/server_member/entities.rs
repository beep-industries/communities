use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::friend::entities::UserId;
use crate::domain::server::entities::ServerId;

/// Unique identifier for a server member
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, ToSchema)]
pub struct MemberId(pub Uuid);

impl Deref for MemberId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for MemberId {
    fn from(uuid: Uuid) -> Self {
        MemberId(uuid)
    }
}

impl From<MemberId> for Uuid {
    fn from(id: MemberId) -> Self {
        id.0
    }
}

impl std::fmt::Display for MemberId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a user's membership in a server
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ServerMember {
    /// Unique member identifier
    pub id: MemberId,
    /// Associated server
    pub server_id: ServerId,
    /// Associated user
    pub user_id: UserId,
    /// Custom nickname in server
    pub nickname: Option<String>,
    /// When member joined
    pub joined_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "postgres")]
impl From<&sqlx::postgres::PgRow> for ServerMember {
    fn from(row: &sqlx::postgres::PgRow) -> Self {
        use sqlx::Row;
        Self {
            id: MemberId(row.get("id")),
            server_id: ServerId(row.get("server_id")),
            user_id: UserId(row.get("user_id")),
            nickname: row.get("nickname"),
            joined_at: row.get("joined_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Input for creating a new server member
#[derive(Debug, Clone, ToSchema)]
pub struct CreateMemberInput {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub nickname: Option<String>,
}

/// Input for updating a server member
#[derive(Debug, Clone, ToSchema)]
pub struct UpdateMemberInput {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub nickname: Option<String>,
}

/// Event emitted when a member is created
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateMemberEvent {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub nickname: Option<String>,
}

/// Event emitted when a member is updated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateMemberEvent {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub nickname: Option<String>,
}

/// Event emitted when a member is deleted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteMemberEvent {
    pub server_id: ServerId,
    pub user_id: UserId,
}
