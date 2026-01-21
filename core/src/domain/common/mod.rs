use serde::Deserialize;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};

use crate::domain::channel::entities::{ChannelError, ChannelId};
use crate::domain::friend::entities::UserId;
use crate::domain::role::entities::RoleId;
use crate::domain::server::entities::ServerId;
use crate::domain::server_member::MemberId;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Error: {msg}")]
    Error { msg: String },

    #[error("Service is currently unavailable")]
    ServiceUnavailable(String),

    #[error("Server with id {id} not found")]
    ServerNotFound { id: ServerId },

    #[error("Failed to insert server with name {name}")]
    FailedToInsertServer { name: String },

    #[error("Server name cannot be empty")]
    InvalidServerName,

    #[error("Failed to manipulate with friendship data")]
    FriendshipDataError,

    #[error("Health check failed")]
    Unhealthy,

    #[error("An unknown error occurred: {message}")]
    UnknownError { message: String },

    #[error("Database error: {msg}")]
    DatabaseError { msg: String },

    /// Serialization error occurred when converting event to JSON
    #[error("Serialization error: {msg}")]
    SerializationError { msg: String },

    #[error("Member already exists for server {server_id} and user {user_id}")]
    MemberAlreadyExists {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Member not found for server {server_id} and user {user_id}")]
    MemberNotFound {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Member not found id = {member_id}")]
    MemberNotFoundById { member_id: MemberId },

    #[error("Invalid member nickname: cannot be empty or whitespace")]
    InvalidMemberNickname,

    #[error("Failed to insert member for server {server_id} and user {user_id}")]
    FailedToInsertMember {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Channel with id {id} not found")]
    ChannelNotFound { id: ChannelId },

    #[error("Channel fields provided are not correctly formatted: {msg}")]
    ChannelPayloadError { msg: String, err: ChannelError },

    #[error("Role with id {id} not found")]
    RoleNotFound { id: RoleId },

    #[error("Role and member are not in the same server")]
    BadRoleMemberAssignation,

    #[error("Could not assign role {role_id} to member {member_id}")]
    AssignMemberRoleError {
        member_id: MemberId,
        role_id: RoleId,
    },

    #[error("Forbidden")]
    Forbidden,

    #[error("Default role basic user cannot be deleted")]
    DefaultRoleDeletion,
}

impl From<ChannelError> for CoreError {
    fn from(value: ChannelError) -> Self {
        match value {
            ChannelError::ChannelNameTooLong
            | ChannelError::WrongChannelType
            | ChannelError::ChannelNameTooShort
            | ChannelError::EmptyUpdatePayload => Self::ChannelPayloadError {
                msg: value.to_string(),
                err: value,
            },
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(default)]
pub struct GetPaginated {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub page: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub limit: u32,
}

fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};
    
    struct StringOrU32;

    impl<'de> serde::de::Visitor<'de> for StringOrU32 {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or u32")
        }

        fn visit_str<E>(self, value: &str) -> Result<u32, E>
        where
            E: de::Error,
        {
            value.parse::<u32>().map_err(de::Error::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<u32, E>
        where
            E: de::Error,
        {
            u32::try_from(value).map_err(de::Error::custom)
        }

        fn visit_u32<E>(self, value: u32) -> Result<u32, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
    }

    deserializer.deserialize_any(StringOrU32)
}

impl Default for GetPaginated {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

pub type TotalPaginatedElements = u64;
