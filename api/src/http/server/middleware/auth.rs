use axum::{
    body::Body,
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::{IntoResponse, Response},
};
use beep_auth::AuthRepository;

use crate::http::server::{AppState, ApiError};

fn extract_token_from_bearer(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = match req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        Some(header) => header,
        None => return ApiError::Unauthorized.into_response(),
    };

    let token = match extract_token_from_bearer(auth_header) {
        Some(token) => token,
        None => return ApiError::Unauthorized.into_response(),
    };

    let identity = match state.auth_repository.identify(token).await {
        Ok(identity) => identity,
        Err(_) => return ApiError::Unauthorized.into_response(),
    };

    req.extensions_mut().insert(identity);

    next.run(req).await
}
