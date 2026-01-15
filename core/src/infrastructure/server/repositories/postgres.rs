use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    domain::{
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        role::entities::{Permission, Permissions, Role},
        server::{
            entities::{DeleteServerEvent, InsertServerInput, Server, ServerId, UpdateServerInput},
            ports::ServerRepository,
        },
        server_member::ServerMember,
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

#[derive(Clone)]
pub struct PostgresServerRepository {
    pub(crate) pool: PgPool,
    delete_server_router: MessageRoutingInfo,
    create_server_router: MessageRoutingInfo,
    create_role_router: MessageRoutingInfo,
}

impl PostgresServerRepository {
    pub fn new(
        pool: PgPool,
        delete_server_router: MessageRoutingInfo,
        create_server_router: MessageRoutingInfo,
        create_role_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            delete_server_router,
            create_server_router,
            create_role_router,
        }
    }
}

impl ServerRepository for PostgresServerRepository {
    async fn find_by_id(&self, id: &ServerId) -> Result<Server, CoreError> {
        let server = query_as!(
            Server,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, 
                   visibility as "visibility: _", created_at, updated_at
            FROM servers
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CoreError::ServerNotFound { id: id.clone() })?;
        match server {
            Some(s) => Ok(s),
            None => Err(CoreError::ServerNotFound { id: id.clone() }),
        }
    }

    async fn list(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;
        let limit = std::cmp::min(pagination.limit, 50) as i64;

        // Get total count of public servers only
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM servers WHERE visibility = 'public'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Get paginated public servers only
        let servers = query_as!(
            Server,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id,
                   visibility as "visibility: _", created_at, updated_at
            FROM servers
            WHERE visibility = 'public'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok((servers, total as u64))
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
            INSERT INTO servers (name, owner_id, picture_url, banner_url, description, visibility)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, banner_url, picture_url, description, owner_id, 
                      visibility as "visibility: _", created_at, updated_at
            "#,
            input.name,
            input.owner_id.0,
            input.picture_url,
            input.banner_url,
            input.description,
            input.visibility as _
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToInsertServer {
            name: input.name.clone(),
        })?;

        let member_id = Uuid::new_v4();
        let _ = query_as!(
            ServerMember,
            r#"
            INSERT INTO server_members (id, server_id, user_id)
            VALUES ($1, $2, $3)
            RETURNING id, server_id, user_id, nickname, joined_at, updated_at
            "#,
            member_id,
            server.id.0,
            input.owner_id.0,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToInsertMember {
            server_id: server.id,
            user_id: input.owner_id,
        })?;
        // Write the create event to the outbox table for eventual processing
        let create_server_event =
            OutboxEventRecord::new(self.create_server_router.clone(), server.clone());
        create_server_event.write(&mut *tx).await?;

        let base_permission = Permissions::from(vec![
            Permission::SendMessages,
            Permission::AttachFiles,
            Permission::ViewChannels,
        ]);

        let role = query_as!(
            Role,
            r#"
            INSERT INTO roles (server_id, name, permissions)
            VALUES ($1, $2, $3)
            RETURNING id, server_id, name, permissions as "permissions: _", created_at, updated_at
            "#,
            *server.id,
            "BasicUser",
            *base_permission
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
        let create_role_event = OutboxEventRecord::new(self.create_role_router.clone(), role);

        create_role_event.write(&mut *tx).await?;

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
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // First, fetch the current server to get existing values
        let current = query_as!(
            Server,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, 
                   visibility as "visibility: _", created_at, updated_at
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
        let new_visibility = input.visibility.as_ref().unwrap_or(&current.visibility);

        // Update the server in the database
        let server = query_as!(
            Server,
            r#"
            UPDATE servers
            SET name = $1, picture_url = $2, banner_url = $3, description = $4, visibility = $5
            WHERE id = $6
            RETURNING id, name, banner_url, picture_url, description, owner_id, 
                      visibility as "visibility: _", created_at, updated_at
            "#,
            new_name,
            new_picture_url,
            new_banner_url,
            new_description,
            new_visibility as _,
            input.id.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::ServerNotFound {
            id: input.id.clone(),
        })?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(server)
    }

    async fn delete(&self, id: &ServerId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Delete the server inside the database
        let result = sqlx::query(r#"DELETE FROM servers WHERE id = $1"#)
            .bind(id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

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
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(())
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn test_insert_server_writes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::{
        friend::entities::UserId,
        server::entities::{InsertServerInput, ServerVisibility},
    };
    use crate::infrastructure::outbox::MessageRouter;
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router.clone(),
        MessageRoutingInfo::default(),
    );

    let owner_id = UserId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "my test server".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("a description".to_string()),
        visibility: ServerVisibility::Public,
    };

    // Act: insert server
    let created = repository.insert(input.clone()).await?;

    // Assert: returned fields
    assert_eq!(created.name, input.name);
    assert_eq!(created.owner_id, owner_id);
    assert_eq!(created.picture_url, input.picture_url);
    assert_eq!(created.banner_url, input.banner_url);
    assert_eq!(created.description, input.description);
    assert_eq!(created.visibility, input.visibility);
    // id should be set and created_at present
    assert!(created.updated_at.is_none());

    // Assert: it can be fetched back
    let fetched = repository.find_by_id(&created.id).await?;
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);

    // Assert: server member was automatically created for the owner
    use sqlx::Row;
    let member_row = sqlx::query(
        r#"
        SELECT server_id, user_id, nickname
        FROM server_members
        WHERE server_id = $1 AND user_id = $2
        "#,
    )
    .bind(created.id.0)
    .bind(owner_id.0)
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let member_server_id: Uuid = member_row
        .try_get("server_id")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let member_user_id: Uuid = member_row
        .try_get("user_id")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let member_nickname: Option<String> = member_row
        .try_get("nickname")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    assert_eq!(member_server_id, created.id.0);
    assert_eq!(member_user_id, owner_id.0);
    assert!(member_nickname.is_none(), "Default nickname should be None");

    // Assert: an outbox message was written with expected routing and payload
    // Note: payload is the serialized InsertServerInput
    let row = sqlx::query(
        r#"
        SELECT exchange_name, payload
        FROM outbox_messages
        WHERE exchange_name = $1 
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(create_router.exchange_name())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, create_router.exchange_name());

    // Validate the payload JSON contains the server name and owner_id
    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(
        payload.get("name").and_then(|v| v.as_str()),
        Some(created.name.as_str())
    );
    // UserId is a newtype around Uuid and serializes to the inner value
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

    let create_router = MessageRoutingInfo::new("server.exchange");
    let delete_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        delete_router,
        create_router,
        MessageRoutingInfo::default(),
    );

    // Try to find a server with a random UUID that doesn't exist
    let nonexistent_id = ServerId(Uuid::new_v4());
    let result = repository.find_by_id(&nonexistent_id).await;

    // Assert: should return error
    assert!(result.is_err());
    match result {
        Err(CoreError::ServerNotFound { id }) => assert_eq!(id, nonexistent_id),
        _ => panic!("Expected ServerNotFound error"),
    }

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_returns_error(pool: PgPool) -> Result<(), CoreError> {
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");
    let delete_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        delete_router,
        create_router,
        MessageRoutingInfo::default(),
    );

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
    use crate::domain::{
        friend::entities::UserId,
        server::entities::{InsertServerInput, ServerVisibility},
    };
    use crate::infrastructure::outbox::MessageRouter;
    use sqlx::Row;
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");
    let delete_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        delete_router.clone(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Arrange: insert a server first
    let owner_id = UserId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "to delete".to_string(),
        owner_id: owner_id.clone(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Private,
    };
    let created = repository.insert(input).await?;

    // Act: delete it
    repository.delete(&created.id).await?;

    // Assert: it's gone - should return error
    let fetched = repository.find_by_id(&created.id).await;
    assert!(fetched.is_err());
    match fetched {
        Err(CoreError::ServerNotFound { .. }) => {}
        _ => panic!("Expected ServerNotFound error"),
    }

    // Assert: an outbox message for delete was written
    let row = sqlx::query(
        r#"
        SELECT exchange_name, payload
        FROM outbox_messages
        WHERE exchange_name = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(delete_router.exchange_name())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, delete_router.exchange_name());

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
async fn test_update_server_updates_fields(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::{
        friend::entities::UserId,
        server::entities::{InsertServerInput, ServerVisibility, UpdateServerInput},
    };
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Arrange: insert a server first
    let owner_id = UserId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "original name".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/old.png".to_string()),
        banner_url: Some("https://example.com/old-banner.png".to_string()),
        description: Some("old description".to_string()),
        visibility: ServerVisibility::Public,
    };
    let created = repository.insert(input).await?;

    // Act: update the server
    let update_input = UpdateServerInput {
        id: created.id.clone(),
        name: Some("updated name".to_string()),
        picture_url: Some("https://example.com/new.png".to_string()),
        banner_url: None,
        description: Some("new description".to_string()),
        visibility: Some(ServerVisibility::Private),
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
    assert_eq!(updated.visibility, ServerVisibility::Private);
    assert!(updated.updated_at.is_some());

    // Assert: it can be fetched back with updates
    let fetched = repository.find_by_id(&created.id).await?;
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

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Try to update a server with a random UUID that doesn't exist
    let nonexistent_id = ServerId(Uuid::new_v4());
    let update_input = UpdateServerInput {
        id: nonexistent_id.clone(),
        name: Some("new name".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
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
    use crate::domain::server::entities::{InsertServerInput, ServerVisibility, UpdateServerInput};
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Arrange: insert a server first
    let owner_id = crate::domain::friend::entities::UserId(Uuid::new_v4());
    let input = InsertServerInput {
        name: "test server".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let created = repository.insert(input).await?;

    // Act: update with no fields
    let update_input = UpdateServerInput {
        id: created.id.clone(),
        name: None,
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };
    let result = repository.update(update_input).await?;

    // Assert: returned server is unchanged
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, created.name);
    assert_eq!(result.picture_url, created.picture_url);

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_servers_with_pagination(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::common::GetPaginated;
    use crate::domain::{
        friend::entities::UserId,
        server::entities::{InsertServerInput, ServerVisibility},
    };
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Arrange: insert multiple servers
    let owner_id = UserId(Uuid::new_v4());
    for i in 1..=5 {
        let input = InsertServerInput {
            name: format!("Server {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Description {}", i)),
            visibility: ServerVisibility::Public,
        };
        repository.insert(input).await?;
    }

    // Act: list first page with 2 items
    let pagination = GetPaginated { page: 1, limit: 2 };
    let (servers, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(servers.len(), 2);
    assert_eq!(servers[0].name, "Server 5"); // Most recent first
    assert_eq!(servers[1].name, "Server 4");

    // Act: list second page
    let pagination = GetPaginated { page: 2, limit: 2 };
    let (servers, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(servers.len(), 2);
    assert_eq!(servers[0].name, "Server 3");
    assert_eq!(servers[1].name, "Server 2");

    // Act: list third page
    let pagination = GetPaginated { page: 3, limit: 2 };
    let (servers, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "Server 1");

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_servers_filters_only_public(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::common::GetPaginated;
    use crate::domain::{
        friend::entities::UserId,
        server::entities::{InsertServerInput, ServerVisibility},
    };
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new("server.exchange");

    let repository = PostgresServerRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router,
        MessageRoutingInfo::default(),
    );

    // Arrange: insert servers with mixed visibility
    let owner_id = UserId(Uuid::new_v4());

    // Create 3 public servers
    for i in 1..=3 {
        let input = InsertServerInput {
            name: format!("Public Server {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Public description {}", i)),
            visibility: ServerVisibility::Public,
        };
        repository.insert(input).await?;
    }

    // Create 2 private servers
    for i in 1..=2 {
        let input = InsertServerInput {
            name: format!("Private Server {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Private description {}", i)),
            visibility: ServerVisibility::Private,
        };
        repository.insert(input).await?;
    }

    // Act: list all servers
    let pagination = GetPaginated { page: 1, limit: 10 };
    let (servers, total) = repository.list(&pagination).await?;

    // Assert: returns only public servers (per security requirements)
    assert_eq!(total, 3, "Should return total count of public servers only");
    assert_eq!(
        servers.len(),
        3,
        "Should return only public servers in the page"
    );

    // Verify all returned servers are public
    let all_public = servers
        .iter()
        .all(|s| s.visibility == ServerVisibility::Public);

    assert!(all_public, "All returned servers should be public");

    // Verify ordering (most recent first)
    assert_eq!(servers[0].name, "Public Server 3");
    assert_eq!(servers[1].name, "Public Server 2");
    assert_eq!(servers[2].name, "Public Server 1");

    Ok(())
}
