use std::future::Future;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::channel::entities::{
    CreateChannelRepoInput, CreatePrivateChannelInput, CreateServerChannelInput, UpdateChannelInput,
};
use crate::domain::{
    channel::entities::{Channel, ChannelId},
    common::CoreError,
    server::entities::ServerId,
};

pub trait ChannelRepository: Send + Sync {
    fn create(
        &self,
        create_channel_input: CreateChannelRepoInput,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
    fn list_in_server(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<Vec<Channel>, CoreError>> + Send;
    fn update(&self, channel: Channel) -> impl Future<Output = Result<Channel, CoreError>> + Send;
    fn delete(&self, channel_id: ChannelId) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn find_by_id(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
}

pub trait ChannelService: Send + Sync {
    fn create_private_channel(
        &self,
        create_channel_input: CreatePrivateChannelInput,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
    fn create_server_channel(
        &self,
        create_channel_input: CreateServerChannelInput,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
    fn list_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<Vec<Channel>, CoreError>> + Send;
    fn update_channel(
        &self,
        update_channel_input: UpdateChannelInput,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
    fn delete_channel(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_channel_by_id(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;
}

/// Mock implementation of ChannelRepository for testing
#[derive(Clone)]
pub struct MockChannelRepository {
    channels: Arc<Mutex<Vec<Channel>>>,
}

impl MockChannelRepository {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_channels(channels: Vec<Channel>) -> Self {
        Self {
            channels: Arc::new(Mutex::new(channels)),
        }
    }
}

impl ChannelRepository for MockChannelRepository {
    async fn create(
        &self,
        create_channel_input: CreateChannelRepoInput,
    ) -> Result<Channel, CoreError> {
        use chrono::Utc;
        let channel = Channel {
            id: ChannelId(Uuid::new_v4()),
            name: create_channel_input.name,
            server_id: create_channel_input.server_id,
            parent_id: create_channel_input.parent_id,
            channel_type: create_channel_input.channel_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut channels = self.channels.lock().unwrap();

        channels.push(channel.clone());
        Ok(channel)
    }

    async fn list_in_server(&self, server_id: ServerId) -> Result<Vec<Channel>, CoreError> {
        let channels = self.channels.lock().unwrap();
        let filtered: Vec<Channel> = channels
            .iter()
            .filter(|c| c.server_id == Some(server_id))
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn update(&self, channel: Channel) -> Result<Channel, CoreError> {
        let mut channels = self.channels.lock().unwrap();
        let existing = channels.iter_mut().find(|c| c.id == channel.id);

        match existing {
            Some(c) => {
                *c = channel.clone();
                Ok(channel)
            }
            None => Err(CoreError::ChannelNotFound { id: channel.id }),
        }
    }

    async fn delete(&self, channel_id: ChannelId) -> Result<(), CoreError> {
        let mut channels = self.channels.lock().unwrap();
        let initial_len = channels.len();
        channels.retain(|c| c.id != channel_id);

        if channels.len() == initial_len {
            Err(CoreError::ChannelNotFound { id: channel_id })
        } else {
            Ok(())
        }
    }

    async fn find_by_id(&self, channel_id: ChannelId) -> Result<Channel, CoreError> {
        let channels = self.channels.lock().unwrap();
        channels
            .iter()
            .find(|c| c.id == channel_id)
            .cloned()
            .ok_or(CoreError::ChannelNotFound { id: channel_id })
    }
}
