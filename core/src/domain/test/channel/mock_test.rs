#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::channel::entities::{Channel, ChannelId, ChannelType};
    use crate::domain::channel::ports::ChannelRepository;
    use crate::domain::common::CoreError;
    use crate::domain::server::entities::ServerId;
    use crate::domain::test::create_mock_service;

    #[tokio::test]
    async fn test_create_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "general".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.channel_repository.create(channel.clone()).await?;

        assert_eq!(created.id, channel.id);
        assert_eq!(created.name, channel.name);
        assert_eq!(created.server_id, channel.server_id);
        assert_eq!(created.channel_type, channel.channel_type);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_id_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
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

        service.channel_repository.create(channel.clone()).await?;

        let found = service.channel_repository.find_by_id(channel_id).await?;

        assert_eq!(found.id, channel.id);
        assert_eq!(found.name, channel.name);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let result = service.channel_repository.find_by_id(ChannelId::from(Uuid::new_v4())).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_in_server_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
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

        service.channel_repository.create(channel1.clone()).await?;
        service.channel_repository.create(channel2.clone()).await?;
        service.channel_repository.create(channel3.clone()).await?;

        let channels = service.channel_repository.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 2);
        assert!(channels.iter().any(|c| c.id == channel1.id));
        assert!(channels.iter().any(|c| c.id == channel2.id));
        assert!(!channels.iter().any(|c| c.id == channel3.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_in_server_empty() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let channels = service.channel_repository.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
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

        service.channel_repository.create(channel.clone()).await?;

        let mut updated_channel = channel.clone();
        updated_channel.name = "new-name".to_string();

        let result = service.channel_repository.update(updated_channel.clone()).await?;

        assert_eq!(result.id, channel_id);
        assert_eq!(result.name, "new-name");

        // Verify it's persisted
        let found = service.channel_repository.find_by_id(channel_id).await?;
        assert_eq!(found.name, "new-name");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "non-existent".to_string(),
            server_id: Some(ServerId::from(Uuid::new_v4())),
            parent_id: None,
            channel_type: ChannelType::ServerText,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.channel_repository.update(channel).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
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

        service.channel_repository.create(channel.clone()).await?;

        // Verify it exists
        let found = service.channel_repository.find_by_id(channel_id).await?;
        assert_eq!(found.id, channel_id);

        // Delete it
        service.channel_repository.delete(channel_id).await?;

        // Verify it's gone
        let result = service.channel_repository.find_by_id(channel_id).await;
        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let result = service.channel_repository.delete(ChannelId::from(Uuid::new_v4())).await;

        assert!(matches!(result, Err(CoreError::ChannelNotFound { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_multiple_channels_in_same_server() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();
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

        service.channel_repository.create(channel1.clone()).await?;
        service.channel_repository.create(channel2.clone()).await?;

        let channels = service.channel_repository.list_in_server(server_id).await?;

        assert_eq!(channels.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_channel_with_parent() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
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

        service.channel_repository.create(parent_channel.clone()).await?;
        let created_child = service.channel_repository.create(child_channel.clone()).await?;

        assert_eq!(created_child.parent_id, Some(parent_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_direct_message_channel() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let dm_channel = Channel {
            id: ChannelId::from(Uuid::new_v4()),
            name: "DM".to_string(),
            server_id: None,
            parent_id: None,
            channel_type: ChannelType::DirectMessage,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = service.channel_repository.create(dm_channel.clone()).await?;

        assert_eq!(created.server_id, None);
        assert_eq!(created.channel_type, ChannelType::DirectMessage);

        Ok(())
    }
}
