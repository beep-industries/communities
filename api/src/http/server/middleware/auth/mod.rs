use axum::{extract::FromRequestParts, http::request::Parts};
use beep_auth::{AuthRepository, KeycloakAuthRepository};
use communities_core::domain::friend::entities::UserId;
use uuid::Uuid;

use crate::http::server::ApiError;
pub mod entities;
pub struct AuthMiddleware;

impl FromRequestParts<KeycloakAuthRepository> for AuthMiddleware {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &KeycloakAuthRepository,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        // Ensure the header exists and starts with "Bearer "
        let token = auth_header
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::Unauthorized)?;

        // Validate the token
        let keycloak_identity = state
            .identify(token)
            .await
            .map_err(|_| ApiError::Unauthorized)?;
        let user_id_uuid =
            Uuid::try_parse(keycloak_identity.id()).map_err(|_| ApiError::Unauthorized)?;
        let user_identity = entities::UserIdentity {
            user_id: UserId(user_id_uuid),
        };

        // Add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
