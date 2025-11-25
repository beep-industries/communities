use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnerId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub banner_url: Option<String>,
    pub picture_url: Option<String>,
    pub description: Option<String>,
    pub owner_id: OwnerId,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertServerInput {
    pub name: String,
    pub owner_id: OwnerId,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
}
