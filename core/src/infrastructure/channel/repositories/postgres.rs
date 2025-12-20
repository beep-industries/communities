use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::{
        channel::{
            entities::{
                Channel, ChannelId, ChannelType, CreateChannelRepoInput, UpdateChannelRepoInput,
            },
            ports::ChannelRepository,
        },
        common::CoreError,
        server::entities::ServerId,
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

/// Event emitted when a channel is created
#[derive(Debug, Clone, Serialize)]
pub struct CreateChannelEvent {
    pub id: ChannelId,
    pub name: String,
    pub server_id: Option<ServerId>,
    pub parent_id: Option<ChannelId>,
    pub channel_type: ChannelType,
}

/// Event emitted when a channel is deleted
#[derive(Debug, Clone, Serialize)]
pub struct DeleteChannelEvent {
    pub id: ChannelId,
    pub server_id: Option<ServerId>,
}

#[derive(Clone)]
pub struct PostgresChannelRepository {
    pub(crate) pool: PgPool,
    create_channel_router: MessageRoutingInfo,
    delete_channel_router: MessageRoutingInfo,
}

impl PostgresChannelRepository {
    pub fn new(
        pool: PgPool,
        create_channel_router: MessageRoutingInfo,
        delete_channel_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            create_channel_router,
            delete_channel_router,
        }
    }
}

/// SQL representation of ChannelType for database queries
#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "channel_type", rename_all = "snake_case")]
enum SqlChannelType {
    ServerText,
    ServerVoice,
    ServerFolder,
    Private,
}

impl From<ChannelType> for SqlChannelType {
    fn from(ct: ChannelType) -> Self {
        match ct {
            ChannelType::ServerText => SqlChannelType::ServerText,
            ChannelType::ServerVoice => SqlChannelType::ServerVoice,
            ChannelType::ServerFolder => SqlChannelType::ServerFolder,
            ChannelType::Private => SqlChannelType::Private,
        }
    }
}

impl From<SqlChannelType> for ChannelType {
    fn from(ct: SqlChannelType) -> Self {
        match ct {
            SqlChannelType::ServerText => ChannelType::ServerText,
            SqlChannelType::ServerVoice => ChannelType::ServerVoice,
            SqlChannelType::ServerFolder => ChannelType::ServerFolder,
            SqlChannelType::Private => ChannelType::Private,
        }
    }
}

/// Internal struct for mapping database rows to Channel entities
struct ChannelRow {
    id: Uuid,
    name: String,
    server_id: Option<Uuid>,
    parent_id: Option<Uuid>,
    channel_type: SqlChannelType,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ChannelRow> for Channel {
    fn from(row: ChannelRow) -> Self {
        Channel {
            id: ChannelId(row.id),
            name: row.name,
            server_id: row.server_id.map(ServerId),
            parent_id: row.parent_id.map(ChannelId),
            channel_type: row.channel_type.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl ChannelRepository for PostgresChannelRepository {
    async fn create(&self, input: CreateChannelRepoInput) -> Result<Channel, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;

        let sql_channel_type: SqlChannelType = input.channel_type.into();
        let server_id = input.server_id.map(|s| s.0);
        let parent_id = input.parent_id.map(|p| p.0);

        let row = sqlx::query_as!(
            ChannelRow,
            r#"
            INSERT INTO channels (name, server_id, parent_id, channel_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            "#,
            input.name,
            server_id,
            parent_id,
            sql_channel_type as SqlChannelType
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to create channel: {}", e),
        })?;

        let channel: Channel = row.into();

        // Only send outbox event for server channels
        if channel.server_id.is_some() {
            let create_event = CreateChannelEvent {
                id: channel.id,
                name: channel.name.clone(),
                server_id: channel.server_id,
                parent_id: channel.parent_id,
                channel_type: channel.channel_type,
            };
            let outbox_event =
                OutboxEventRecord::new(self.create_channel_router.clone(), create_event);
            outbox_event.write(&mut *tx).await?;
        }

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(channel)
    }

    async fn list_in_server(&self, server_id: ServerId) -> Result<Vec<Channel>, CoreError> {
        let rows = sqlx::query_as!(
            ChannelRow,
            r#"
            SELECT id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            FROM channels
            WHERE server_id = $1
            ORDER BY created_at ASC
            "#,
            server_id.0
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to list channels: {}", e),
        })?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, input: UpdateChannelRepoInput) -> Result<Channel, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;

        // First, fetch the current channel to get existing values
        let current = sqlx::query_as!(
            ChannelRow,
            r#"
            SELECT id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            FROM channels
            WHERE id = $1
            "#,
            input.id.0
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to find channel: {}", e),
        })?
        .ok_or_else(|| CoreError::ChannelNotFound { id: input.id })?;

        // Apply updates, falling back to current values if not provided
        let new_name = input.name.as_ref().unwrap_or(&current.name);
        let new_parent_id = match input.parent_id {
            Some(pid) => Some(pid.0),
            None => current.parent_id,
        };

        // Update the channel in the database
        let row = sqlx::query_as!(
            ChannelRow,
            r#"
            UPDATE channels
            SET name = $1, parent_id = $2
            WHERE id = $3
            RETURNING id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            "#,
            new_name,
            new_parent_id,
            input.id.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to update channel: {}", e),
        })?;

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(row.into())
    }

    async fn delete(&self, channel_id: ChannelId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;

        // First, get the channel to know if it's a server channel (for outbox event)
        let channel = sqlx::query_as!(
            ChannelRow,
            r#"
            SELECT id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            FROM channels
            WHERE id = $1
            "#,
            channel_id.0
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to find channel: {}", e),
        })?
        .ok_or_else(|| CoreError::ChannelNotFound { id: channel_id })?;

        // Delete the channel
        let result = sqlx::query(r#"DELETE FROM channels WHERE id = $1"#)
            .bind(channel_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to delete channel: {}", e),
            })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::ChannelNotFound { id: channel_id });
        }

        // Only send outbox event for server channels
        if channel.server_id.is_some() {
            let delete_event = DeleteChannelEvent {
                id: channel_id,
                server_id: channel.server_id.map(ServerId),
            };
            let outbox_event =
                OutboxEventRecord::new(self.delete_channel_router.clone(), delete_event);
            outbox_event.write(&mut *tx).await?;
        }

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(())
    }

    async fn find_by_id(&self, channel_id: ChannelId) -> Result<Channel, CoreError> {
        let row = sqlx::query_as!(
            ChannelRow,
            r#"
            SELECT id, name, server_id, parent_id, channel_type as "channel_type: SqlChannelType", created_at, updated_at
            FROM channels
            WHERE id = $1
            "#,
            channel_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to find channel: {}", e),
        })?;

        match row {
            Some(r) => Ok(r.into()),
            None => Err(CoreError::ChannelNotFound { id: channel_id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper function to create a test server
    async fn create_test_server(pool: &PgPool, server_id: ServerId) -> Result<(), CoreError> {
        use crate::domain::server::entities::ServerVisibility;

        sqlx::query(
            r#"
            INSERT INTO servers (id, name, owner_id, visibility)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(server_id.0)
        .bind("Test Server")
        .bind(Uuid::new_v4())
        .bind(ServerVisibility::Public)
        .execute(pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to create test server: {}", e),
        })?;
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_server_channel_writes_row_and_outbox(
        pool: PgPool,
    ) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        let input = CreateChannelRepoInput {
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        // Act: create channel
        let created = repository.create(input).await?;

        // Assert: returned fields
        assert_eq!(created.name, "general");
        assert_eq!(created.server_id, Some(server_id));
        assert_eq!(created.parent_id, None);
        assert_eq!(created.channel_type, ChannelType::ServerText);

        // Assert: it can be fetched back
        let fetched = repository.find_by_id(created.id).await?;
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);

        // Assert: outbox event was written
        let outbox_row = sqlx::query(
            r#"
                SELECT exchange_name, routing_key, payload, status
                FROM outbox_messages
                WHERE routing_key = 'channel.created'
                "#,
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to query outbox: {}", e),
        })?;

        assert!(outbox_row.is_some(), "Outbox event should be written");
        let row = outbox_row.unwrap();
        let status: String = row.get("status");
        assert_eq!(status, "READY");

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_private_channel_no_outbox(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository = PostgresChannelRepository::new(pool.clone(), create_router, delete_router);

        let input = CreateChannelRepoInput {
            name: "my-dm".to_string(),
            server_id: None,
            parent_id: None,
            channel_type: ChannelType::Private,
        };

        // Act: create private channel
        let created = repository.create(input).await?;

        // Assert: channel created
        assert_eq!(created.name, "my-dm");
        assert_eq!(created.server_id, None);
        assert_eq!(created.channel_type, ChannelType::Private);

        // Assert: no outbox event for private channels
        let outbox_count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM outbox_messages WHERE routing_key = 'channel.created'"#,
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to query outbox: {}", e),
        })?;

        assert_eq!(outbox_count, 0, "No outbox event for private channels");

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_channels_in_server(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        // Create multiple channels
        let input1 = CreateChannelRepoInput {
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };
        let input2 = CreateChannelRepoInput {
            name: "voice-chat".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerVoice,
        };

        repository.create(input1).await?;
        repository.create(input2).await?;

        // Act: list channels
        let channels = repository.list_in_server(server_id).await?;

        // Assert: both channels returned
        assert_eq!(channels.len(), 2);
        let names: Vec<&str> = channels.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"general"));
        assert!(names.contains(&"voice-chat"));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_channel(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        let input = CreateChannelRepoInput {
            name: "old-name".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = repository.create(input).await?;

        // Act: update channel
        let update_input = UpdateChannelRepoInput {
            id: created.id,
            name: Some("new-name".to_string()),
            parent_id: None,
        };

        let updated = repository.update(update_input).await?;

        // Assert: name changed
        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.id, created.id);

        // Verify via find_by_id
        let fetched = repository.find_by_id(created.id).await?;
        assert_eq!(fetched.name, "new-name");

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_server_channel_writes_outbox(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        let input = CreateChannelRepoInput {
            name: "to-delete".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerText,
        };

        let created = repository.create(input).await?;

        // Act: delete channel
        repository.delete(created.id).await?;

        // Assert: channel no longer exists
        let result = repository.find_by_id(created.id).await;
        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { id }) => assert_eq!(id, created.id),
            _ => panic!("Expected ChannelNotFound error"),
        }

        // Assert: outbox event was written for delete
        let outbox_row = sqlx::query(
            r#"
            SELECT exchange_name, routing_key, payload, status
            FROM outbox_messages
            WHERE routing_key = 'channel.deleted'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to query outbox: {}", e),
        })?;

        assert!(
            outbox_row.is_some(),
            "Delete outbox event should be written"
        );

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_nonexistent_channel_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository = PostgresChannelRepository::new(pool.clone(), create_router, delete_router);

        let nonexistent_id = ChannelId(Uuid::new_v4());

        // Act: try to delete non-existent channel
        let result = repository.delete(nonexistent_id).await;

        // Assert: error returned
        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected ChannelNotFound error"),
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_channel_parent_child_relationship(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        // Create parent folder
        let folder_input = CreateChannelRepoInput {
            name: "Category".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let folder = repository.create(folder_input).await?;
        assert_eq!(folder.channel_type, ChannelType::ServerFolder);

        // Create child channel under the folder
        let child_input = CreateChannelRepoInput {
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: Some(folder.id),
            channel_type: ChannelType::ServerText,
        };

        let child = repository.create(child_input).await?;

        // Assert: child has correct parent
        assert_eq!(child.parent_id, Some(folder.id));
        assert_eq!(child.server_id, Some(server_id));

        // Fetch and verify relationship
        let fetched_child = repository.find_by_id(child.id).await?;
        assert_eq!(fetched_child.parent_id, Some(folder.id));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_channel_parent(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository =
            PostgresChannelRepository::new(pool.clone(), create_router.clone(), delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        // Create two folders
        let folder1_input = CreateChannelRepoInput {
            name: "Category 1".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };
        let folder2_input = CreateChannelRepoInput {
            name: "Category 2".to_string(),
            server_id: Some(server_id),
            parent_id: None,
            channel_type: ChannelType::ServerFolder,
        };

        let folder1 = repository.create(folder1_input).await?;
        let folder2 = repository.create(folder2_input).await?;

        // Create channel under folder1
        let channel_input = CreateChannelRepoInput {
            name: "general".to_string(),
            server_id: Some(server_id),
            parent_id: Some(folder1.id),
            channel_type: ChannelType::ServerText,
        };

        let channel = repository.create(channel_input).await?;
        assert_eq!(channel.parent_id, Some(folder1.id));

        // Act: move channel to folder2
        let update_input = UpdateChannelRepoInput {
            id: channel.id,
            name: None,
            parent_id: Some(folder2.id),
        };

        let updated = repository.update(update_input).await?;

        // Assert: parent changed
        assert_eq!(updated.parent_id, Some(folder2.id));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_nonexistent_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository = PostgresChannelRepository::new(pool.clone(), create_router, delete_router);

        let nonexistent_id = ChannelId(Uuid::new_v4());

        // Act: try to find non-existent channel
        let result = repository.find_by_id(nonexistent_id).await;

        // Assert: error returned
        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected ChannelNotFound error"),
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_nonexistent_channel_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository = PostgresChannelRepository::new(pool.clone(), create_router, delete_router);

        let nonexistent_id = ChannelId(Uuid::new_v4());

        // Act: try to update non-existent channel
        let update_input = UpdateChannelRepoInput {
            id: nonexistent_id,
            name: Some("new-name".to_string()),
            parent_id: None,
        };

        let result = repository.update(update_input).await;

        // Assert: error returned
        assert!(result.is_err());
        match result {
            Err(CoreError::ChannelNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected ChannelNotFound error"),
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_channels_in_empty_server(pool: PgPool) -> Result<(), CoreError> {
        let create_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.created".to_string(),
        );
        let delete_router = MessageRoutingInfo::new(
            "channel.exchange".to_string(),
            "channel.deleted".to_string(),
        );

        let repository = PostgresChannelRepository::new(pool.clone(), create_router, delete_router);

        let server_id = ServerId(Uuid::new_v4());
        create_test_server(&pool, server_id).await?;

        // Act: list channels in server with no channels
        let channels = repository.list_in_server(server_id).await?;

        // Assert: empty list returned
        assert!(channels.is_empty());

        Ok(())
    }
}
