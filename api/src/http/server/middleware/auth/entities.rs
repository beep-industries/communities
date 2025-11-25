use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::server::ApiError;

#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub exp: i64,  // expiration timestamp
    pub iat: i64,  // issued at timestamp
}

impl Claims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}

#[derive(Clone)]
pub struct AuthValidator {
    secret_key: String,
}

impl AuthValidator {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}

pub trait TokenValidator: Send + Sync {
    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError>;
}

impl TokenValidator for AuthValidator {
    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?;

        let claims = token_data.claims;

        // check if the token has expired
        if claims.is_expired() {
            return Err(ApiError::Unauthorized);
        }

        Ok(UserIdentity {
            user_id: claims.sub,
        })
    }
}
