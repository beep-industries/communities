use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ServerId(pub Uuid);

impl std::fmt::Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ServerId {
    fn from(uuid: Uuid) -> Self {
        ServerId(uuid)
    }
}

impl From<ServerId> for Uuid {
    fn from(server_id: ServerId) -> Self {
        server_id.0
    }
}

impl From<Uuid> for OwnerId {
    fn from(uuid: Uuid) -> Self {
        OwnerId(uuid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct OwnerId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type, Default, ToSchema)]
#[sqlx(type_name = "server_visibility", rename_all = "lowercase")]
pub enum ServerVisibility {
    #[default]
    Public,
    Private,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub banner_url: Option<String>,
    pub picture_url: Option<String>,
    pub description: Option<String>,
    pub owner_id: OwnerId,
    pub visibility: ServerVisibility,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InsertServerInput {
    pub name: String,
    pub owner_id: OwnerId,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: ServerVisibility,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateServerRequest {
    pub name: String,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: ServerVisibility,
}

impl CreateServerRequest {
    pub fn into_input(self, owner_id: OwnerId) -> InsertServerInput {
        InsertServerInput {
            name: self.name,
            owner_id,
            picture_url: self.picture_url,
            banner_url: self.banner_url,
            description: self.description,
            visibility: self.visibility,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateServerInput {
    pub id: ServerId,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<ServerVisibility>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<ServerVisibility>,
}

impl UpdateServerRequest {
    pub fn into_input(self, id: ServerId) -> UpdateServerInput {
        UpdateServerInput {
            id,
            name: self.name,
            picture_url: self.picture_url,
            banner_url: self.banner_url,
            description: self.description,
            visibility: self.visibility,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateServerEvent {
    pub id: ServerId,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<ServerVisibility>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteServerEvent {
    pub id: ServerId,
}
