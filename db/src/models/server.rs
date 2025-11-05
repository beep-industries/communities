use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub owner_id: String,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub banner_url: Option<String>,
    pub picture_url: Option<String>,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateServer {
    pub name: Option<String>,
    pub banner_url: Option<String>,
    pub picture_url: Option<String>,
    pub description: Option<String>,
}
