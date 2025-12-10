use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::server::entities::ServerId;

pub const MAX_CHANNEL_NAME_SIZE: usize = 30;

pub const MIN_CHANNEL_NAME_SIZE: usize = 2;

#[derive(Error, Debug, Clone)]
pub enum ChannelError {
    #[error(
        "Channel name is too long. It should not be longer than {}",
        MAX_CHANNEL_NAME_SIZE
    )]
    ChannelNameTooLong,

    #[error(
        "Channel name is too short. It should not be  than {}",
        MIN_CHANNEL_NAME_SIZE
    )]
    ChannelNameTooShort,

    #[error("Channel type is incorrect")]
    WrongChannelType,

    #[error("The payload to update the channel is empty")]
    EmptyUpdatePayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChannelId(pub Uuid);

/// The string is the value of the name
/// The Option<bool> represetant if the state of the validation
/// If the validation is not already
#[derive(Clone, Debug)]
pub struct ChannelName(String, Option<bool>);

impl ChannelName {
    pub fn new(value: String) -> Self {
        let formated_value = value.trim().to_string();
        Self(formated_value, None)
    }

    fn is_too_long(&self) -> bool {
        self.0.len() > MAX_CHANNEL_NAME_SIZE
    }

    fn is_too_short(&self) -> bool {
        self.0.len() < MIN_CHANNEL_NAME_SIZE
    }

    pub fn check(&mut self) -> Result<(), ChannelError> {
        if self.is_too_long() {
            self.1 = Some(false);
            return Err(ChannelError::ChannelNameTooLong);
        }
        if self.is_too_short() {
            self.1 = Some(false);
            return Err(ChannelError::ChannelNameTooShort);
        }
        Ok(())
    }

    pub fn is_valid(&mut self) -> bool {
        if let Some(is_valid) = self.1 {
            is_valid
        } else {
            self.check().is_err()
        }
    }

    pub fn value(&mut self) -> Result<String, ChannelError> {
        self.clone().check()?;
        Ok(self.clone().0)
    }
}

impl From<Uuid> for ChannelId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub server_id: Option<ServerId>,
    pub parent_id: Option<ChannelId>,
    pub channel_type: ChannelType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    ServerText,
    ServerVoice,
    ServerFolder,
    Private,
}

pub struct CreatePrivateChannelInput {
    pub name: ChannelName,
}

pub struct CreateServerChannelInput {
    pub name: ChannelName,
    pub server_id: ServerId,
    pub parent_id: Option<ChannelId>,
    pub channel_type: ChannelType,
}

#[derive(Default, Clone)]
pub struct UpdateChannelInput {
    pub id: ChannelId,
    pub name: Option<ChannelName>,
    pub parent_id: Option<ChannelId>,
}

impl UpdateChannelInput {
    // Check if all element are none
    fn is_empty(self) -> bool {
        self.name.is_none() && self.parent_id.is_none()
    }

    pub fn into_repo_input(&mut self) -> Result<UpdateChannelRepoInput, ChannelError> {
        if self.clone().is_empty() {
            return Err(ChannelError::EmptyUpdatePayload.into());
        }

        let mut repo_input = UpdateChannelRepoInput {
            id: self.id,
            parent_id: self.parent_id,
            ..Default::default()
        };

        repo_input.name = if let Some(mut channel_name) = self.name.clone() {
            Some(channel_name.value()?)
        } else {
            None
        };

        Ok(repo_input)
    }
}

pub struct CreateChannelRepoInput {
    pub name: String,
    pub server_id: Option<ServerId>,
    pub parent_id: Option<ChannelId>,
    pub channel_type: ChannelType,
}

#[derive(Default)]
pub struct UpdateChannelRepoInput {
    pub id: ChannelId,
    pub name: Option<String>,
    pub parent_id: Option<ChannelId>,
}
