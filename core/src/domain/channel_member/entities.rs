use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelMember {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct CreateChannelMemberInput {
    pub user_id: Uuid,
    pub channel_id: Uuid,
}

pub struct DeleteChannelMemberInput {
    pub user_id: Uuid,
    pub channel_id: Uuid,
}
