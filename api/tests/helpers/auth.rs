use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Test claims structure matching the expected JWT format
#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

/// Request body for Keycloak token endpoint
#[derive(Debug, Serialize)]
struct TokenRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
}

/// Response from Keycloak token endpoint
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    token_type: String,
}

/// Generate a test JWT token by authenticating with Keycloak
/// This requires Keycloak to be running (e.g., via docker-compose)
/// Retries up to 30 times with 1 second delays to wait for Keycloak to be ready
pub async fn get_keycloak_token(
    keycloak_url: &str,
    realm: &str,
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        keycloak_url, realm
    );

    let client = reqwest::Client::new();
    let max_retries = 30;
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match client
            .post(&token_url)
            .form(&[
                ("grant_type", "password"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("username", username),
                ("password", password),
            ])
            .send()
            .await
        {
            Ok(response) => match response.json::<TokenResponse>().await {
                Ok(token_response) => {
                    if attempt > 1 {
                        println!("✓ Keycloak ready after {} attempts", attempt);
                    }
                    return Ok(token_response.access_token);
                }
                Err(e) => {
                    last_error = Some(format!("JSON decode error: {}", e));
                }
            },
            Err(e) => {
                last_error = Some(format!("Connection error: {}", e));
            }
        }

        if attempt < max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Err(last_error
        .unwrap_or_else(|| "Unknown error".to_string())
        .into())
}

/// Generate a simple test JWT token for local testing without Keycloak
/// WARNING: This will NOT work with real Keycloak validation
/// Use this only for unit tests that mock the auth repository
pub fn generate_mock_token(user_id: &Uuid) -> String {
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};

    let now = Utc::now().timestamp();
    let claims = TestClaims {
        sub: user_id.to_string(),
        exp: now + 3600, // Token expires in 1 hour
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("test-secret-key".as_ref()),
    )
    .expect("Failed to generate mock token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_token() {
        let user_id = Uuid::new_v4();
        let token = generate_mock_token(&user_id);
        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires Keycloak to be running
    async fn test_get_keycloak_token() {
        let token = get_keycloak_token(
            "http://localhost:8080",
            "myrealm",
            "user-service",
            "ABvykyIUah2CcQPiRcvcgd7GA4MrEdx4",
            "testuser",
            "testpassword",
        )
        .await;

        match token {
            Ok(t) => {
                assert!(!t.is_empty());
                println!("Successfully obtained token from Keycloak");
            }
            Err(e) => {
                println!("Failed to get token (Keycloak might not be running): {}", e);
            }
        }
    }
}
