use axum::{extract::FromRequestParts, http::request::Parts};
use beep_auth::{AuthRepository, KeycloakAuthRepository};

use crate::http::server::ApiError;

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
        let user_identity = state
            .validate_token(token)
            .await
            .map_err(|_| ApiError::Unauthorized)?;

        // Add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
