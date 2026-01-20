use reqwest::{self, StatusCode};

use crate::{
    domain::user::{entities::User, port::UserRepository},
    infrastructure::user::repositories::error::UserError,
};

use tracing::{error, info};
use urlencoding::encode;

#[derive(Clone)]
pub struct HttpUserRepository {
    base_url: String,
    client: reqwest::Client,
}

impl HttpUserRepository {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

impl UserRepository for HttpUserRepository {
    async fn get_user_by_username(&self, username: &String) -> Result<Option<User>, UserError> {
        info!("Fetching user by username: {}", username);
        let res = match self
            .client
            .get(format!("{}/users/username/{}", self.base_url, encode(username)))
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("An error occurred with the User service: {}", e);
                return Err(UserError::UserNotFound);
            }
        };

        info!("Received response with status: {}", res.status());

        if res.status() != StatusCode::OK {
            if res.status() == StatusCode::NOT_FOUND {
                return Ok(None);
            } else {
                error!(
                    "User service returned an unexpected status code: {}",
                    res.status()
                );
                return Err(UserError::UserNotFound);
            }
        }

        let user = match res.json::<User>().await {
            Ok(user) => Some(user),
            Err(e) => {
                error!("Failed to deserialize user: {}", e);
                return Err(UserError::UserNotFound);
            }
        };

        Ok(user)
    }
}
