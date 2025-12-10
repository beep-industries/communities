#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::channel::entities::{Channel, ChannelId, ChannelType};
    use crate::domain::channel::ports::{ChannelRepository, MockChannelRepository};
    use crate::domain::common::CoreError;
    use crate::domain::server::entities::ServerId;

    #[tokio::test]
    async fn test_create_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();

        let channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "general".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = repo.create(channel.clone()).await?;

        assert_eq!(created.id, channel.id);
        assert_eq!(created.name, channel.name);
        assert_eq!(created.server_id, channel.server_id);
        assert_eq!(created.channel_type, channel.channel_type);

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_find_by_id_success() -> Result<(), Box<dyn std::error::Error>> {
        let channel_id = ChannelId::from(Uuid::new_v4());
        let channel = Channel {
            id: channel_id,
            name: "announcements".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = MockChannelRepository::with_channels(vec![channel.clone()]);

        let found = repo.find_by_id(channel_id).await?;

        assert_eq!(found.id, channel.id);
        assert_eq!(found.name, channel.name);

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_find_by_id_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();

        let result = repo.find_by_id(ChannelId::from(Uuid::new_v4())).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_list_in_server_success() -> Result<(), Box<dyn std::error::Error>> {
        let server_id = ServerId::from(Uuid::new_v4());
        let other_server_id = ServerId::from(Uuid::new_v4());

        let channel1 = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let channel2 = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "voice-1".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerVoice,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let channel3 = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "other-server-channel".to_string(),
            server_id: Some(other_server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = MockChannelRepository::with_channels(vec![
            channel1.clone(),
            channel2.clone(),
            channel3.clone(),
        ]);

        let channels = repo.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 2);
        assert!(channels.iter().any(|c| c.id == channel1.id));
        assert!(channels.iter().any(|c| c.id == channel2.id));
        assert!(!channels.iter().any(|c| c.id == channel3.id));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_list_in_server_empty() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();
        let server_id = ServerId::from(Uuid::new_v4());

        let channels = repo.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 0);

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_update_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let channel_id = ChannelId::from(Uuid::new_v4());
        let channel = Channel {
            id: channel_id,
            name: "old-name".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = MockChannelRepository::with_channels(vec![channel.clone()]);

        let mut updated_channel = channel.clone();
        updated_channel.name = "new-name".to_string();

        let result = repo.update(updated_channel.clone()).await?;

        assert_eq!(result.id, channel_id);
        assert_eq!(result.name, "new-name");

        // Verify it's persisted
        let found = repo.find_by_id(channel_id).await?;
        assert_eq!(found.name, "new-name");

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_update_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();

        let channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "non-existent".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = repo.update(channel).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_delete_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let channel_id = ChannelId::from(Uuid::new_v4());
        let channel = Channel {
            id: channel_id,
            name: "to-delete".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = MockChannelRepository::with_channels(vec![channel.clone()]);

        // Verify it exists
        let found = repo.find_by_id(channel_id).await?;
        assert_eq!(found.id, channel_id);

        // Delete it
        repo.delete(channel_id).await?;

        // Verify it's gone
        let result = repo.find_by_id(channel_id).await;
        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_delete_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();

        let result = repo.delete(ChannelId::from(Uuid::new_v4())).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_create_multiple_channels_in_same_server() -> Result<(), Box<dyn std::error::Error>>
    {
        let repo = MockChannelRepository::new();
        let server_id = ServerId::from(Uuid::new_v4());

        let channel1 = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let channel2 = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "voice-1".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerVoice,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create(channel1.clone()).await?;
        repo.create(channel2.clone()).await?;

        let channels = repo.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 2);

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_channel_with_parent() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();
        let server_id = ServerId::from(Uuid::new_v4());
        let parent_id = ChannelId::from(Uuid::new_v4());

        let parent_channel = Channel {
            id: parent_id,
            name: "Category".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let child_channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "text-channel".to_string(),
            server_id: Some(server_id),
            parent_id: Some(parent_id),
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create(parent_channel.clone()).await?;
        let created_child = repo.create(child_channel.clone()).await?;

        assert_eq!(created_child.parent_id, Some(parent_id));

        Ok(())
    }

    #[tokio::test]
    #[cfg(test)]
    async fn test_direct_message_channel() -> Result<(), Box<dyn std::error::Error>> {
        let repo = MockChannelRepository::new();

        let dm_channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "DM".to_string(),
            server_id: None,
            parent_id: None,
            channel_type: ChannelType::DirectMessage,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = repo.create(dm_channel.clone()).await?;

        assert_eq!(created.server_id, None);
        assert_eq!(created.channel_type, ChannelType::DirectMessage);

        Ok(())
    }
}
