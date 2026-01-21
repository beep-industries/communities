use std::ops::Deref;

use chrono::{DateTime, Utc};
use events_protobuf::communities_events::{CreateServer, DeleteServer};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{common::GetPaginated, friend::entities::UserId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ServerId(pub Uuid);

impl std::fmt::Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ServerId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    pub owner_id: UserId,
    pub visibility: ServerVisibility,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Server {
    pub fn is_public(&self) -> bool {
        self.visibility == ServerVisibility::Public
    }
}

impl Into<CreateServer> for Server {
    fn into(self) -> CreateServer {
        CreateServer {
            server_id: self.id.0.to_string(),
            owner_id: self.owner_id.0.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InsertServerInput {
    pub name: String,
    pub owner_id: UserId,
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
    pub fn into_input(self, owner_id: UserId) -> InsertServerInput {
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

impl Into<DeleteServer> for DeleteServerEvent {
    fn into(self) -> DeleteServer {
        DeleteServer {
            server_id: self.id.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchServerQuery {
    #[serde(rename = "q")]
    pub query: Option<String>,
    #[serde(flatten)]
    pub pagination: GetPaginated,
}

impl SearchServerQuery {
    const MAX_QUERY_LENGTH: usize = 100;
    const MAX_LIMIT: u32 = 50;

    /// Validates and sanitizes the search query
    pub fn sanitized_query(&self) -> Option<String> {
        self.query.as_ref().and_then(|q| {
            let trimmed = q.trim();
            
            // Reject empty or too long queries
            if trimmed.is_empty() || trimmed.len() > Self::MAX_QUERY_LENGTH {
                return None;
            }

            // Remove any null bytes or control characters
            let cleaned: String = trimmed
                .chars()
                .filter(|c| !c.is_control() && *c != '\0')
                .collect();

            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        })
    }

    /// Returns a safe pagination with enforced max limit
    pub fn safe_pagination(&self) -> GetPaginated {
        GetPaginated {
            page: self.pagination.page,
            limit: self.pagination.limit.min(Self::MAX_LIMIT),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitized_query_normal_input() {
        let query = SearchServerQuery {
            query: Some("gaming server".to_string()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), Some("gaming server".to_string()));
    }

    #[test]
    fn test_sanitized_query_removes_control_characters() {
        let query = SearchServerQuery {
            query: Some("gaming\x00\x01\x02server".to_string()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), Some("gamingserver".to_string()));
    }

    #[test]
    fn test_sanitized_query_trims_whitespace() {
        let query = SearchServerQuery {
            query: Some("  gaming  ".to_string()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), Some("gaming".to_string()));
    }

    #[test]
    fn test_sanitized_query_rejects_empty() {
        let query = SearchServerQuery {
            query: Some("".to_string()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), None);
    }

    #[test]
    fn test_sanitized_query_rejects_whitespace_only() {
        let query = SearchServerQuery {
            query: Some("   ".to_string()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), None);
    }

    #[test]
    fn test_sanitized_query_rejects_too_long() {
        let long_query = "a".repeat(150);
        let query = SearchServerQuery {
            query: Some(long_query),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), None);
    }

    #[test]
    fn test_sanitized_query_accepts_max_length() {
        let max_query = "a".repeat(100);
        let query = SearchServerQuery {
            query: Some(max_query.clone()),
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), Some(max_query));
    }

    #[test]
    fn test_sanitized_query_none_returns_none() {
        let query = SearchServerQuery {
            query: None,
            pagination: GetPaginated { page: 1, limit: 20 },
        };
        
        assert_eq!(query.sanitized_query(), None);
    }

    #[test]
    fn test_safe_pagination_enforces_max_limit() {
        let query = SearchServerQuery {
            query: Some("test".to_string()),
            pagination: GetPaginated { page: 1, limit: 100 },
        };
        
        let safe = query.safe_pagination();
        assert_eq!(safe.limit, 50);
        assert_eq!(safe.page, 1);
    }

    #[test]
    fn test_safe_pagination_preserves_lower_limit() {
        let query = SearchServerQuery {
            query: Some("test".to_string()),
            pagination: GetPaginated { page: 2, limit: 20 },
        };
        
        let safe = query.safe_pagination();
        assert_eq!(safe.limit, 20);
        assert_eq!(safe.page, 2);
    }

    #[test]
    fn test_safe_pagination_exactly_max_limit() {
        let query = SearchServerQuery {
            query: Some("test".to_string()),
            pagination: GetPaginated { page: 1, limit: 50 },
        };
        
        let safe = query.safe_pagination();
        assert_eq!(safe.limit, 50);
    }
}
