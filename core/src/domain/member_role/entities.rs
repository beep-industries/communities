use chrono::{DateTime, Utc};
use events_protobuf::communities_events::{MemberAssignedToRole, MemberRemovedFromRole};
use serde::{Deserialize, Serialize};

use crate::domain::{friend::entities::UserId, role::entities::RoleId, server_member::MemberId};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignUserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}

impl Into<MemberAssignedToRole> for AssignUserRole {
    fn into(self) -> MemberAssignedToRole {
        MemberAssignedToRole {
            user_id: self.user_id.to_string(),
            role_id: self.role_id.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct UnassignMemberRole {
    pub member_id: MemberId,
    pub role_id: RoleId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnassignUserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}

impl Into<MemberRemovedFromRole> for UnassignUserRole {
    fn into(self) -> MemberRemovedFromRole {
        MemberRemovedFromRole {
            user_id: self.user_id.to_string(),
            role_id: self.user_id.to_string(),
        }
    }
}
