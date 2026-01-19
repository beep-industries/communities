use std::ops::Deref;

use chrono::Utc;
use communities_core::{
    CommunitiesService,
    domain::{
        authorization::ports::AuthorizationService, common::CoreError, friend::entities::UserId,
        server::entities::ServerId, server_member::MemberService,
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserIdentity {
    service: CommunitiesService,
    pub user_id: UserId,
}

impl UserIdentity {
    pub fn new(service: CommunitiesService, user_id_uuid: Uuid) -> Self {
        Self {
            service,
            user_id: UserId(user_id_uuid),
        }
    }

    pub async fn can_view_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> Result<bool, CoreError> {
        self.service
            .can_view_channels_in_server(self.user_id, server_id)
            .await
    }

    pub async fn can_manage_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> Result<bool, CoreError> {
        self.service
            .can_manage_channels_in_server(self.user_id, server_id)
            .await
    }

    pub async fn can_manage_server(&self, server_id: ServerId) -> Result<bool, CoreError> {
        self.service
            .can_manage_server(self.user_id, server_id)
            .await
    }

    pub async fn can_view_server(&self, server_id: ServerId) -> Result<bool, CoreError> {
        let _ = self
            .service
            .get_member(server_id, self.user_id)
            .await
            .map_err(|_| CoreError::Forbidden)?;
        Ok(true)
    }

    pub async fn can_manage_role_in_servers(&self, server_id: ServerId) -> Result<bool, CoreError> {
        self.service
            .can_manage_roles_in_server(self.user_id, server_id)
            .await
    }
}

impl Deref for UserIdentity {
    type Target = UserId;

    fn deref(&self) -> &Self::Target {
        &self.user_id
    }
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
