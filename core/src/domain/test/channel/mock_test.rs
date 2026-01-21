#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::channel::entities::{
        ChannelName, ChannelType, CreatePrivateChannelInput, CreateServerChannelInput,
        UpdateChannelInput,
    };
    use crate::domain::channel::ports::ChannelService;
    use crate::domain::common::CoreError;
    use crate::domain::server::entities::ServerId;
    use crate::domain::test::create_mock_service;

    #[tokio::test]
    async fn test_create_private_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let input = CreatePrivateChannelInput {
            name: ChannelName::new("Direct Message".to_string()),
        };

        let created = service.create_private_channel(input).await?;

        assert_eq!(created.name, "Direct Message");
        assert_eq!(created.server_id, None);
        assert_eq!(created.channel_type, ChannelType::Private);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_private_channel_with_whitespace() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();

        let input = CreatePrivateChannelInput {
            name: ChannelName::new("  Trimmed Name  ".to_string()),
        };

        let created = service.create_private_channel(input).await?;

        assert_eq!(created.name, "Trimmed Name");
        assert_eq!(created.channel_type, ChannelType::Private);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_private_channel_name_too_long() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let input = CreatePrivateChannelInput {
            name: ChannelName::new(
                "This is a very long channel name that exceeds the maximum limit".to_string(),
            ),
        };

        let result = service.create_private_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("Channel name is too long"));
            }
            _ => {
                panic!("Expected CoreError::ChannelPayloadError with channel name too long message")
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: ChannelName::new("general".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(input).await?;

        assert_eq!(created.name, "general");
        assert_eq!(created.server_id, Some(server_id));
        assert_eq!(created.channel_type, ChannelType::ServerText);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_with_whitespace() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: ChannelName::new("  voice channel  ".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerVoice,
        };

        let created = service.create_server_channel(input).await?;

        assert_eq!(created.name, "voice channel");
        assert_eq!(created.channel_type, ChannelType::ServerVoice);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_name_too_long() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: ChannelName::new(
                "This is a very long channel name that exceeds the maximum limit".to_string(),
            ),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let result = service.create_server_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("Channel name is too long"));
            }
            _ => {
                panic!("Expected CoreError::ChannelPayloadError with channel name too long message")
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_wrong_type() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: ChannelName::new("wrong-type".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::Private,
        };

        let result = service.create_server_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("Channel type is incorrect"));
            }
            _ => panic!("Expected CoreError::ChannelPayloadError with wrong channel type message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_with_folder_type() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: ChannelName::new("Category".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let created = service.create_server_channel(input).await?;

        assert_eq!(created.name, "Category");
        assert_eq!(created.channel_type, ChannelType::ServerFolder);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_name_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel first
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("old-name".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Update the channel name
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: Some(ChannelName::new("new-name".to_string())),
            parent_id: None,
        };

        let updated = service.update_channel(update_input).await?;

        assert_eq!(updated.id, channel_id);
        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.server_id, Some(server_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_parent_id() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a parent folder
        let parent_input = CreateServerChannelInput {
            name: ChannelName::new("Category".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let parent = service.create_server_channel(parent_input).await?;
        let parent_id = parent.id;

        // Create a channel
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("general".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Update the channel to have a parent
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: None,
            parent_id: Some(parent_id),
        };

        let updated = service.update_channel(update_input).await?;

        assert_eq!(updated.id, channel_id);
        assert_eq!(updated.parent_id, Some(parent_id));
        assert_eq!(updated.name, "general"); // Name should remain unchanged

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_both_name_and_parent() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a parent folder
        let parent_input = CreateServerChannelInput {
            name: ChannelName::new("Category".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let parent = service.create_server_channel(parent_input).await?;
        let parent_id = parent.id;

        // Create a channel
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("old-name".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Update both name and parent
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: Some(ChannelName::new("new-name".to_string())),
            parent_id: Some(parent_id),
        };

        let updated = service.update_channel(update_input).await?;

        assert_eq!(updated.id, channel_id);
        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.parent_id, Some(parent_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_empty_payload() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel first
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("test-channel".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Try to update with empty payload
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: None,
            parent_id: None,
        };

        let result = service.update_channel(update_input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected CoreError::ChannelPayloadError with empty payload message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_name_too_long() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel first
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("test-channel".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Try to update with a name that's too long
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: Some(ChannelName::new(
                "This is a very long channel name that exceeds the maximum limit".to_string(),
            )),
            parent_id: None,
        };

        let result = service.update_channel(update_input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("Channel name is too long"));
            }
            _ => panic!("Expected CoreError::ChannelPayloadError with name too long message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_name_too_short() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel first
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("test-channel".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Try to update with a name that's too short
        let update_input = UpdateChannelInput {
            id: channel_id,
            name: Some(ChannelName::new("a".to_string())),
            parent_id: None,
        };

        let result = service.update_channel(update_input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelPayloadError { msg, .. }) => {
                assert!(msg.contains("Channel name is too short"));
            }
            _ => panic!("Expected CoreError::ChannelPayloadError with name too short message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Try to update a non-existent channel
        let update_input = UpdateChannelInput {
            id: crate::domain::channel::entities::ChannelId::from(Uuid::new_v4()),
            name: Some(ChannelName::new("new-name".to_string())),
            parent_id: None,
        };

        let result = service.update_channel(update_input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { .. }) => {
                // Expected error
            }
            _ => panic!("Expected CoreError::ChannelNotFound"),
        }

        Ok(())
    }

    // Tests for get_channel_by_id
    #[tokio::test]
    async fn test_get_channel_by_id_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("test-channel".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Get the channel by ID
        let found = service.get_channel_by_id(channel_id).await?;

        assert_eq!(found.id, channel_id);
        assert_eq!(found.name, "test-channel");
        assert_eq!(found.server_id, Some(server_id));
        assert_eq!(found.channel_type, ChannelType::ServerText);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_channel_by_id_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let result = service
            .get_channel_by_id(crate::domain::channel::entities::ChannelId::from(
                Uuid::new_v4(),
            ))
            .await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { .. }) => {
                // Expected error
            }
            _ => panic!("Expected CoreError::ChannelNotFound"),
        }

        Ok(())
    }

    // Tests for list_channels_in_server
    #[tokio::test]
    async fn test_list_channels_in_server_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());
        let other_server_id = ServerId::from(Uuid::new_v4());

        // Create multiple channels in the same server
        let input1 = CreateServerChannelInput {
            name: ChannelName::new("general".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let input2 = CreateServerChannelInput {
            name: ChannelName::new("announcements".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let input3 = CreateServerChannelInput {
            name: ChannelName::new("voice".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerVoice,
        };

        // Create a channel in a different server
        let input4 = CreateServerChannelInput {
            name: ChannelName::new("other-server".to_string()),
            server_id: other_server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let channel1 = service.create_server_channel(input1).await?;
        let channel2 = service.create_server_channel(input2).await?;
        let channel3 = service.create_server_channel(input3).await?;
        let channel4 = service.create_server_channel(input4).await?;

        // List channels in the first server
        let channels = service.list_channels_in_server(server_id).await?;

        assert_eq!(channels.len(), 3);
        assert!(channels.iter().any(|c| c.id == channel1.id));
        assert!(channels.iter().any(|c| c.id == channel2.id));
        assert!(channels.iter().any(|c| c.id == channel3.id));
        assert!(!channels.iter().any(|c| c.id == channel4.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_channels_in_server_empty() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let channels = service.list_channels_in_server(server_id).await?;

        assert_eq!(channels.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_channels_in_server_with_parent() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a parent folder
        let parent_input = CreateServerChannelInput {
            name: ChannelName::new("Category".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let parent = service.create_server_channel(parent_input).await?;
        let parent_id = parent.id;

        // Create a child channel
        let child_input = CreateServerChannelInput {
            name: ChannelName::new("general".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let child = service.create_server_channel(child_input).await?;

        // Update child to have parent
        let update_input = UpdateChannelInput {
            id: child.id,
            name: None,
            parent_id: Some(parent_id),
        };

        service.update_channel(update_input).await?;

        // List all channels in server
        let channels = service.list_channels_in_server(server_id).await?;

        assert_eq!(channels.len(), 2);
        let child_channel = channels.iter().find(|c| c.id == child.id).unwrap();
        assert_eq!(child_channel.parent_id, Some(parent_id));

        Ok(())
    }

    // Tests for delete_channel
    #[tokio::test]
    async fn test_delete_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create a channel
        let create_input = CreateServerChannelInput {
            name: ChannelName::new("to-delete".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = service.create_server_channel(create_input).await?;
        let channel_id = created.id;

        // Verify it exists
        let found = service.get_channel_by_id(channel_id).await?;
        assert_eq!(found.id, channel_id);

        // Delete it
        service.delete_channel(channel_id).await?;

        // Verify it's gone
        let result = service.get_channel_by_id(channel_id).await;
        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { .. }) => {
                // Expected error
            }
            _ => panic!("Expected CoreError::ChannelNotFound"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_channel_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let result = service
            .delete_channel(crate::domain::channel::entities::ChannelId::from(
                Uuid::new_v4(),
            ))
            .await;

        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { .. }) => {
                // Expected error
            }
            _ => panic!("Expected CoreError::ChannelNotFound"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_channel_from_server_list() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        // Create multiple channels
        let input1 = CreateServerChannelInput {
            name: ChannelName::new("channel-1".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let input2 = CreateServerChannelInput {
            name: ChannelName::new("channel-2".to_string()),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let channel1 = service.create_server_channel(input1).await?;
        let channel2 = service.create_server_channel(input2).await?;

        // Verify both exist
        let channels = service.list_channels_in_server(server_id).await?;
        assert_eq!(channels.len(), 2);

        // Delete one channel
        service.delete_channel(channel1.id).await?;

        // Verify only one remains
        let channels = service.list_channels_in_server(server_id).await?;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].id, channel2.id);

        Ok(())
    }
}
