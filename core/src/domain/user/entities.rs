use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub sub: Uuid,
    pub display_name: String,
    pub profile_picture: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserByDisplayname {
    pub display_name: String,
}
