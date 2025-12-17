use chrono::{DateTime, Utc};
use uuid::Uuid;

pub type RoleId = Uuid;

pub struct Role {
    pub id: RoleId,
    pub server_id: Uuid,
    pub name: String,
    pub permissions: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct CreateRoleInput {
    pub server_id: Uuid,
    pub name: String,
    pub permissions: i32,
}

pub struct UpdateRoleInput {
    pub id: RoleId,
    pub name: Option<String>,
    pub permissions: Option<i32>,
}
