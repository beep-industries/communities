use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::http::server::{ApiError, middleware::auth::entities::TokenValidator};
pub mod entities;

pub struct AuthMiddleware;

impl<AuthValidator> FromRequestParts<AuthValidator> for AuthMiddleware
where
    AuthValidator: Send + Sync + TokenValidator,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthValidator,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized)?;
        let auth_cookie = cookie_jar.get("access_token");

        let token = auth_cookie.ok_or_else(|| ApiError::Unauthorized)?.value();
        let user_identity = state.validate_token(token)?;

        // add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
