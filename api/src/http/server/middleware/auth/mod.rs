use axum::{extract::FromRequestParts, http::request::Parts};
use beep_auth::AuthRepository;
use uuid::Uuid;

use crate::http::server::{ApiError, middleware::auth::auth_state::AuthState};
pub mod auth_state;
pub mod entities;
pub struct AuthMiddleware;

impl FromRequestParts<AuthState> for AuthMiddleware {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        // Ensure the header exists and starts with "Bearer "
        let token = auth_header
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        // Validate the token
        let keycloak_identity = state
            .keycloak()
            .identify(token)
            .await
            .map_err(|_| ApiError::Unauthorized)?;
        let user_id_uuid =
            Uuid::try_parse(keycloak_identity.id()).map_err(|_| ApiError::Unauthorized)?;
        let user_identity = entities::UserIdentity::new(state.service(), user_id_uuid);
        // Add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
