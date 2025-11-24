use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::server::ApiError;

#[derive(Clone, Debug)]
pub struct AuthState {
    pub user_id: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}

impl Claims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}

pub async fn auth_middleware(
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // try to get token from cookies
    let auth_cookie = cookie_jar
        .get("access_token");

    let token: String;

    if auth_cookie.is_none() {
        return Err(ApiError::AuthenticationError("Missing JWT token".to_string()));
    } else {
        // extract token from cookie
        token = auth_cookie.unwrap().value().to_string();
    }

    // decode and validate JWT token
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret("Key-Must-Be-at-least-32-bytes-in-length".to_string().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::AuthenticationError("Invalid JWT token".to_string()))?;

    let claims = token_data.claims;

    // check if the token has expired
    if claims.is_expired() {
        return Err(ApiError::AuthenticationError("JWT token expired".to_string()));
    }

    // create auth state
    let auth_state = AuthState {
        user_id: claims.sub
    };

    // add auth state to request
    request.extensions_mut().insert(auth_state);

    Ok(next.run(request).await)
}
