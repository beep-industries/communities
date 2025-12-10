#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::channel::entities::{
        ChannelType, CreatePrivateChannelInput, CreateServerChannelInput,
    };
    use crate::domain::channel::ports::ChannelService;
    use crate::domain::common::CoreError;
    use crate::domain::server::entities::ServerId;
    use crate::domain::test::create_mock_service;

    #[tokio::test]
    async fn test_create_private_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let input = CreatePrivateChannelInput {
            name: "Direct Message".to_string(),
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
            name: "  Trimmed Name  ".to_string(),
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
            name: "This is a very long channel name that exceeds the maximum limit".to_string(),
        };

        let result = service.create_private_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::CreationFailure { msg }) => {
                assert!(msg.contains("Channel name is too long"));
            }
            _ => panic!("Expected CoreError::CreationFailure with channel name too long message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: "general".to_string(),
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
            name: "  voice channel  ".to_string(),
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
            name: "This is a very long channel name that exceeds the maximum limit".to_string(),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let result = service.create_server_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::CreationFailure { msg }) => {
                assert!(msg.contains("Channel name is too long"));
            }
            _ => panic!("Expected CoreError::CreationFailure with channel name too long message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_wrong_type() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: "wrong-type".to_string(),
            server_id,
            parent_id: None,
            channel_type: ChannelType::Private,
        };

        let result = service.create_server_channel(input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::CreationFailure { msg }) => {
                assert!(msg.contains("Channel type is incorrect"));
            }
            _ => panic!("Expected CoreError::CreationFailure with wrong channel type message"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_server_channel_with_folder_type() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();
        let server_id = ServerId::from(Uuid::new_v4());

        let input = CreateServerChannelInput {
            name: "Category".to_string(),
            server_id,
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let created = service.create_server_channel(input).await?;

        assert_eq!(created.name, "Category");
        assert_eq!(created.channel_type, ChannelType::ServerFolder);

        Ok(())
    }
}
