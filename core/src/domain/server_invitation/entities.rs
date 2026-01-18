use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{friend::entities::UserId, server::entities::ServerId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ServerInvitationId(pub Uuid);

impl std::fmt::Display for ServerInvitationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ServerInvitationId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for ServerInvitationId {
    fn from(uuid: Uuid) -> Self {
        ServerInvitationId(uuid)
    }
}

impl From<ServerInvitationId> for Uuid {
    fn from(invitation_id: ServerInvitationId) -> Self {
        invitation_id.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "server_invitation_status", rename_all = "lowercase")]
pub enum ServerInvitationStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ServerInvitation {
    pub id: ServerInvitationId,
    pub server_id: ServerId,
    pub inviter_id: UserId,
    pub invitee_id: Option<UserId>,
    pub status: ServerInvitationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ServerInvitation {
    pub fn is_expired(&self) -> bool {
        if let Some(expiration_date) = self.expires_at {
            let now = Utc::now();
            return if expiration_date > now { false } else { true };
        } else {
            true
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InsertServerInvitationInput {
    pub server_id: ServerId,
    pub inviter_id: UserId,
    pub invitee_id: Option<UserId>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateServerInvitationRequest {
    pub invitee_id: Option<UserId>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CreateServerInvitationRequest {
    pub fn into_input(
        self,
        server_id: ServerId,
        inviter_id: UserId,
    ) -> InsertServerInvitationInput {
        InsertServerInvitationInput {
            server_id,
            inviter_id,
            invitee_id: self.invitee_id,
            expires_at: self.expires_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateServerInvitationInput {
    pub id: ServerInvitationId,
    pub status: ServerInvitationStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AcceptInvitationInput {
    pub user_id: UserId,
    pub invitation_id: ServerInvitationId,
}
