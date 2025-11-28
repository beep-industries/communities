use sqlx::{PgPool, query_as};

use crate::{
    domain::{
        common::CoreError,
        server::{
            entities::{DeleteServerEvent, InsertServerInput, Server, ServerId},
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
