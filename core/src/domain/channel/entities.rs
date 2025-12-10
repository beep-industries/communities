use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::server::entities::ServerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub Uuid);

impl From<Uuid> for ChannelId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub server_id: Option<ServerId>,
    pub parent_id: Option<ChannelId>,
    pub channel_type: ChannelType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    ServerText,
    ServerVoice,
    ServerFolder,
    DirectMessage,
}
