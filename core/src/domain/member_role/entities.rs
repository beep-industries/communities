use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::{role::entities::RoleId, server_member::MemberId};

#[derive(Debug, Serialize, Clone)]
pub struct MemberRole {
    pub member_id: MemberId,
    pub role_id: RoleId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AssignMemberRole {
    pub member_id: MemberId,
    pub role_id: RoleId,
}

#[derive(Debug, Serialize, Clone)]
pub struct UnassignMemberRole {
    pub member_id: MemberId,
    pub role_id: RoleId,
}
