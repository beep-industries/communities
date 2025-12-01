use sqlx::{PgPool, query_as};

use crate::{
    domain::{
        common::CoreError,
        server::{
            entities::{DeleteServerEvent, InsertServerInput, Server, ServerId, UpdateServerInput},
            ports::ServerRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

#[derive(Clone)]
pub struct PostgresServerRepository {
    pub(crate) pool: PgPool,
    delete_server_router: MessageRoutingInfo,
    create_server_router: MessageRoutingInfo,
}

impl PostgresServerRepository {
    pub fn new(
        pool: PgPool,
        delete_server_router: MessageRoutingInfo,
        create_server_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            delete_server_router,
            create_server_router,
        }
    }
}

impl ServerRepository for PostgresServerRepository {
    async fn find_by_id(&self, id: &ServerId) -> Result<Option<Server>, CoreError> {
        let server = query_as!(
            Server,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, created_at, updated_at
            FROM servers
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CoreError::ServerNotFound { id: id.clone() })?;

        Ok(server)
    }

    async fn insert(&self, input: InsertServerInput) -> Result<Server, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| CoreError::FailedToInsertServer {
                name: input.name.clone(),
            })?;

        // Insert the server into the database
        let server = query_as!(
            Server,
            r#"
            INSERT INTO servers (name, owner_id, picture_url, banner_url, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, banner_url, picture_url, description, owner_id, created_at, updated_at
            "#,
            input.name,
            input.owner_id.0,
            input.picture_url,
            input.banner_url,
            input.description
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToInsertServer { name: input.name.clone() })?;

        // Write the create event to the outbox table for eventual processing
        let create_server_event =
            OutboxEventRecord::new(self.create_server_router.clone(), input.clone());
        create_server_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|_| CoreError::FailedToInsertServer { name: input.name })?;

        Ok(server)
    }

    async fn update(&self, input: UpdateServerInput) -> Result<Server, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| CoreError::ServerNotFound {
                id: input.id.clone(),
            })?;

        // First, fetch the current server to get existing values
        let current = query_as!(
            Server,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, created_at, updated_at
            FROM servers
            WHERE id = $1
            "#,
            input.id.0
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| CoreError::ServerNotFound {
            id: input.id.clone(),
        })?
        .ok_or_else(|| CoreError::ServerNotFound {
            id: input.id.clone(),
        })?;

        // Apply updates, falling back to current values if not provided
        let new_name = input.name.as_ref().unwrap_or(&current.name);
        let new_picture_url = input.picture_url.as_ref().or(current.picture_url.as_ref());
        let new_banner_url = input.banner_url.as_ref().or(current.banner_url.as_ref());
        let new_description = input.description.as_ref().or(current.description.as_ref());

        // Update the server in the database
        let server = query_as!(
            Server,
            r#"
            UPDATE servers
            SET name = $1, picture_url = $2, banner_url = $3, description = $4
            WHERE id = $5
            RETURNING id, name, banner_url, picture_url, description, owner_id, created_at, updated_at
            "#,
            new_name,
            new_picture_url,
            new_banner_url,
            new_description,
            input.id.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::ServerNotFound { id: input.id.clone() })?;

        tx.commit().await.map_err(|_| CoreError::ServerNotFound {
            id: input.id.clone(),
        })?;

        Ok(server)
    }

    async fn delete(&self, id: &ServerId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| CoreError::ServerNotFound { id: id.clone() })?;

        // Delete the server inside the database
        let result = sqlx::query(r#"DELETE FROM servers WHERE id = $1"#)
            .bind(id.0)
            .execute(&mut *tx)
            .await
            .map_err(|_| CoreError::ServerNotFound { id: id.clone() })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::ServerNotFound { id: id.clone() });
        }

        // Write the delete event to the outbox table
        // for eventual processing
        let event = DeleteServerEvent { id: id.clone() };
        let delete_server_event = OutboxEventRecord::new(self.delete_server_router.clone(), event);
        delete_server_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|_| CoreError::ServerNotFound { id: id.clone() })?;

        Ok(())
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn test_insert_server_writes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::server::entities::{InsertServerInput, OwnerId};
    use crate::infrastructure::outbox::MessageRouter;
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router.clone(),
    );

    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "my test server".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("a description".to_string()),
    };

    // Act: insert server
    let created = repository.insert(input.clone()).await?;

    // Assert: returned fields
    assert_eq!(created.name, input.name);
    assert_eq!(created.owner_id, owner_id);
    assert_eq!(created.picture_url, input.picture_url);
    assert_eq!(created.banner_url, input.banner_url);
    assert_eq!(created.description, input.description);
    // id should be set and created_at present
    assert!(created.updated_at.is_none());

    // Assert: it can be fetched back
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);

    // Assert: an outbox message was written with expected routing and payload
    // Note: payload is the serialized InsertServerInput
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT exchange_name, routing_key, payload
        FROM outbox_messages
        WHERE exchange_name = $1 AND routing_key = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(create_router.exchange_name())
    .bind(create_router.routing_key())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let routing_key: String = row
        .try_get("routing_key")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, create_router.exchange_name());
    assert_eq!(routing_key, create_router.routing_key());

    // Validate the payload JSON contains the server name and owner_id
    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(
        payload.get("name").and_then(|v| v.as_str()),
        Some(created.name.as_str())
    );
    // OwnerId is a newtype around Uuid and serializes to the inner value
    let owner_str = owner_id.0.to_string();
    assert_eq!(
        payload.get("owner_id").and_then(|v| v.as_str()),
        Some(owner_str.as_str())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_find_by_id_returns_none_for_nonexistent(pool: PgPool) -> Result<(), CoreError> {
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());
    let delete_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.deleted".to_string());

    let repository = PostgresServerRepository::new(pool.clone(), delete_router, create_router);

    // Try to find a server with a random UUID that doesn't exist
    let nonexistent_id = ServerId(Uuid::new_v4());
    let result = repository.find_by_id(&nonexistent_id).await?;

    // Assert: should return None
    assert!(result.is_none());

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_returns_error(pool: PgPool) -> Result<(), CoreError> {
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());
    let delete_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.deleted".to_string());

    let repository = PostgresServerRepository::new(pool.clone(), delete_router, create_router);

    // Try to delete a server with a random UUID that doesn't exist
    let nonexistent_id = ServerId(Uuid::new_v4());
    let result = repository.delete(&nonexistent_id).await;

    // Assert: should return ServerNotFound error
    assert!(result.is_err());
    match result {
        Err(CoreError::ServerNotFound { id }) => {
            assert_eq!(id, nonexistent_id);
        }
        _ => panic!("Expected ServerNotFound error"),
    }

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_server_removes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::server::entities::{InsertServerInput, OwnerId};
    use crate::infrastructure::outbox::MessageRouter;
    use sqlx::Row;
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());
    let delete_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.deleted".to_string());

    let repository =
        PostgresServerRepository::new(pool.clone(), delete_router.clone(), create_router);

    // Arrange: insert a server first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "to delete".to_string(),
        owner_id: owner_id.clone(),
        picture_url: None,
        banner_url: None,
        description: None,
    };
    let created = repository.insert(input).await?;

    // Act: delete it
    repository.delete(&created.id).await?;

    // Assert: it's gone
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_none());

    // Assert: an outbox message for delete was written
    let row = sqlx::query(
        r#"
        SELECT exchange_name, routing_key, payload
        FROM outbox_messages
        WHERE routing_key = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(delete_router.routing_key())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let routing_key: String = row
        .try_get("routing_key")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, delete_router.exchange_name());
    assert_eq!(routing_key, delete_router.routing_key());

    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    // Payload should be { "id": "<uuid>" }
    let id_str = created.id.0.to_string();
    assert_eq!(
        payload.get("id").and_then(|v| v.as_str()),
        Some(id_str.as_str())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_server_updates_fields_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::server::entities::{InsertServerInput, OwnerId, UpdateServerInput};
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());

    let repository =
        PostgresServerRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert a server first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "original name".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/old.png".to_string()),
        banner_url: Some("https://example.com/old-banner.png".to_string()),
        description: Some("old description".to_string()),
    };
    let created = repository.insert(input).await?;

    // Act: update the server
    let update_input = UpdateServerInput {
        id: created.id.clone(),
        name: Some("updated name".to_string()),
        picture_url: Some("https://example.com/new.png".to_string()),
        banner_url: None,
        description: Some("new description".to_string()),
    };
    let updated = repository.update(update_input.clone()).await?;

    // Assert: returned server has updated fields
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "updated name");
    assert_eq!(
        updated.picture_url,
        Some("https://example.com/new.png".to_string())
    );
    assert_eq!(
        updated.banner_url,
        Some("https://example.com/old-banner.png".to_string())
    ); // unchanged
    assert_eq!(updated.description, Some("new description".to_string()));
    assert!(updated.updated_at.is_some());

    // Assert: it can be fetched back with updates
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.name, "updated name");
    assert_eq!(
        fetched.picture_url,
        Some("https://example.com/new.png".to_string())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_nonexistent_server_returns_error(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::server::entities::UpdateServerInput;
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());

    let repository =
        PostgresServerRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Try to update a server with a random UUID that doesn't exist
    let nonexistent_id = ServerId(Uuid::new_v4());
    let update_input = UpdateServerInput {
        id: nonexistent_id.clone(),
        name: Some("new name".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
    };
    let result = repository.update(update_input).await;

    // Assert: should return ServerNotFound error
    assert!(result.is_err());
    match result {
        Err(CoreError::ServerNotFound { id }) => {
            assert_eq!(id, nonexistent_id);
        }
        _ => panic!("Expected ServerNotFound error"),
    }

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_server_with_no_fields_returns_unchanged(
    pool: PgPool,
) -> Result<(), CoreError> {
    use crate::domain::server::entities::{InsertServerInput, OwnerId, UpdateServerInput};
    use uuid::Uuid;

    let create_router =
        MessageRoutingInfo::new("server.exchange".to_string(), "server.created".to_string());

    let repository =
        PostgresServerRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert a server first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "test server".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: None,
        description: None,
    };
    let created = repository.insert(input).await?;

    // Act: update with no fields
    let update_input = UpdateServerInput {
        id: created.id.clone(),
        name: None,
        picture_url: None,
        banner_url: None,
        description: None,
    };
    let result = repository.update(update_input).await?;

    // Assert: returned server is unchanged
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, created.name);
    assert_eq!(result.picture_url, created.picture_url);

    Ok(())
}
