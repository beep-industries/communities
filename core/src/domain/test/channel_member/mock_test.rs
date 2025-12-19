mod tests {
    use crate::domain::channel_member::entities::ChannelMember;
    use crate::domain::channel_member::ports::{
        ChannelMemberRepository, MockChannelMemberRepository,
    };
    use crate::domain::common::CoreError;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_channel_member_success() {
        let repository = MockChannelMemberRepository {
            members: Arc::new(Mutex::new(Vec::new())),
        };

        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let result = repository.create(user_id, channel_id).await;

        assert!(result.is_ok());
        let member = result.unwrap();
        assert_eq!(member.user_id, user_id);
        assert_eq!(member.channel_id, channel_id);
    }

    #[tokio::test]
    async fn test_create_channel_member_conflict() {
        let repository = MockChannelMemberRepository {
            members: Arc::new(Mutex::new(Vec::new())),
        };

        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        // First creation should succeed
        repository.create(user_id, channel_id).await.unwrap();

        // Second creation should fail
        let result = repository.create(user_id, channel_id).await;

        assert!(result.is_err());
        if let CoreError::Error { msg } = result.unwrap_err() {
            assert_eq!(msg, "Conflict: Channel member already exists");
        }
    }

    #[tokio::test]
    async fn test_delete_channel_member_success() {
        let repository = MockChannelMemberRepository {
            members: Arc::new(Mutex::new(Vec::new())),
        };

        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        // Add a member
        repository.create(user_id, channel_id).await.unwrap();

        // Delete the member
        let result = repository.delete(user_id, channel_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_channel_member_not_found() {
        let repository = MockChannelMemberRepository {
            members: Arc::new(Mutex::new(Vec::new())),
        };

        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        // Attempt to delete a non-existent member
        let result = repository.delete(user_id, channel_id).await;

        assert!(result.is_err());
        if let CoreError::Error { msg } = result.unwrap_err() {
            assert_eq!(msg, "Not Found: Channel member does not exist");
        }
    }
}
