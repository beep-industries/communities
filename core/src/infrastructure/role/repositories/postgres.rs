use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    domain::{
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        role::{
            entities::{CreateRoleInput, DeleteRole, Role, RoleId, UpdateRoleRepoInput},
            ports::RoleRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

#[derive(Clone)]
pub struct PostgresRoleRepository {
    pool: PgPool,
    create_role_router: MessageRoutingInfo,
    update_role_router: MessageRoutingInfo,
    delete_role_router: MessageRoutingInfo,
}

impl PostgresRoleRepository {
    pub fn new(
        pool: PgPool,
        create_role_router: MessageRoutingInfo,
        update_role_router: MessageRoutingInfo,
        delete_role_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            create_role_router,
            update_role_router,
            delete_role_router,
        }
    }
}

impl RoleRepository for PostgresRoleRepository {
    async fn create(&self, input: CreateRoleInput) -> Result<Role, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let role = query_as!(
            Role,
            r#"
            INSERT INTO roles (server_id, name, permissions)
            VALUES ($1, $2, $3)
            RETURNING id, server_id, name, permissions as "permissions: _", created_at, updated_at
            "#,
            input.server_id,
            input.name,
            input.permissions.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Write the create event to the outbox table for eventual processing
        let create_role_event =
            OutboxEventRecord::new(self.create_role_router.clone(), role.clone());
        create_role_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(role)
    }

    async fn find_by_id(&self, id: &RoleId) -> Result<Role, CoreError> {
        let role = query_as!(
            Role,
            r#"
            SELECT id, server_id, name, permissions as "permissions: _", created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        match role {
            Some(r) => Ok(r),
            None => Err(CoreError::RoleNotFound { id: *id }),
        }
    }

    async fn list_by_server(
        &self,
        pagination: &GetPaginated,
        server_id: Uuid,
    ) -> Result<(Vec<Role>, TotalPaginatedElements), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;
        let limit = std::cmp::min(pagination.limit, 50) as i64;

        // Get total count for the server
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE server_id = $1")
            .bind(server_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Get paginated roles for the server
        let roles = query_as!(
            Role,
            r#"
            SELECT id, server_id, name, permissions as "permissions: _", created_at, updated_at
            FROM roles
            WHERE server_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            server_id,
            limit,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok((roles, total as u64))
    }

    async fn update(&self, input: UpdateRoleRepoInput) -> Result<Role, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // First, fetch the current role to get existing values
        let current = query_as!(
            Role,
            r#"
            SELECT id, server_id, name, permissions as "permissions: _", created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
            input.id.0
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?
        .ok_or_else(|| CoreError::RoleNotFound { id: input.id })?;

        // Apply updates, falling back to current values if not provided
        let new_name = input.name.as_ref().unwrap_or(&current.name);
        let new_permissions = input.permissions.as_ref().unwrap_or(&current.permissions);

        // Update the role in the database
        let role = query_as!(
            Role,
            r#"
            UPDATE roles
            SET name = $1, permissions = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, server_id, name, permissions as "permissions: _", created_at, updated_at
            "#,
            new_name,
            new_permissions.0,
            input.id.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Write the update event to the outbox table for eventual processing
        let update_role_event =
            OutboxEventRecord::new(self.update_role_router.clone(), role.clone());
        update_role_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(role)
    }

    async fn delete(&self, id: &RoleId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let result = sqlx::query(r#"DELETE FROM roles WHERE id = $1"#)
            .bind(id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::RoleNotFound { id: *id });
        }

        // Write the delete event to the outbox table for eventual processing
        let delete_role_event =
            OutboxEventRecord::new(self.delete_role_router.clone(), DeleteRole { role_id: *id });
        delete_role_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::role::entities::Permissions;
    use crate::domain::server::entities::ServerVisibility;
    use crate::infrastructure::outbox::MessageRouter;
    use sqlx::Row;

    async fn create_test_server(pool: &PgPool, name: &str) -> Uuid {
        let owner_id = Uuid::new_v4();
        let server_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO servers (name, owner_id, visibility)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .bind(ServerVisibility::Private)
        .fetch_one(pool)
        .await
        .unwrap();
        server_id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_role_writes_row(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "Admin".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };

        let role = repo.create(input).await.unwrap();

        assert_eq!(*role.server_id, server_id);
        assert_eq!(role.name, "Admin");
        assert_eq!(role.permissions.0, 0x1);
        assert!(role.created_at.timestamp() > 0);
        assert!(role.updated_at.is_none());

        // Verify we can fetch it
        let fetched = repo.find_by_id(&role.id).await.unwrap();
        assert_eq!(fetched.id, role.id);
        assert_eq!(fetched.name, role.name);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_role_writes_outbox(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test.exchange");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo = PostgresRoleRepository::new(
            pool.clone(),
            create_router.clone(),
            update_router,
            delete_router,
        );
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "Admin".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };

        let _role = repo.create(input.clone()).await.unwrap();

        // Assert: an outbox message was written with expected routing and payload
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
        .unwrap();

        let exchange_name: String = row.try_get("exchange_name").unwrap();
        assert_eq!(exchange_name, create_router.exchange_name());

        // Validate the payload JSON contains the role data
        let payload: serde_json::Value = row.try_get("payload").unwrap();
        assert_eq!(payload.get("name").and_then(|v| v.as_str()), Some("Admin"));
        assert_eq!(
            payload.get("server_id").and_then(|v| v.as_str()),
            Some(server_id.to_string().as_str())
        );
        assert_eq!(
            payload.get("permissions").and_then(|v| v.as_i64()),
            Some(0x1)
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_returns_error_for_nonexistent(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo = PostgresRoleRepository::new(pool, create_router, update_router, delete_router);
        let nonexistent_id = RoleId(Uuid::new_v4());

        let result = repo.find_by_id(&nonexistent_id).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected RoleNotFound error"),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_by_server_with_pagination(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server_id = create_test_server(&pool, "Test Server").await;

        // Create 5 roles
        for i in 1..=5 {
            let input = CreateRoleInput {
                server_id,
                name: format!("Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            repo.create(input).await.unwrap();
        }

        // Test page 1 with limit 3
        let pagination1 = GetPaginated { page: 1, limit: 3 };
        let (roles1, total1) = repo.list_by_server(&pagination1, server_id).await.unwrap();
        assert_eq!(roles1.len(), 3);
        assert_eq!(total1, 5);

        // Test page 2 with limit 3
        let pagination2 = GetPaginated { page: 2, limit: 3 };
        let (roles2, total2) = repo.list_by_server(&pagination2, server_id).await.unwrap();
        assert_eq!(roles2.len(), 2);
        assert_eq!(total2, 5);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_by_server_filters_by_server_id(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server1_id = create_test_server(&pool, "Server 1").await;
        let server2_id = create_test_server(&pool, "Server 2").await;

        // Create roles for server 1
        for i in 1..=3 {
            let input = CreateRoleInput {
                server_id: server1_id,
                name: format!("Server1 Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            repo.create(input).await.unwrap();
        }

        // Create roles for server 2
        for i in 1..=2 {
            let input = CreateRoleInput {
                server_id: server2_id,
                name: format!("Server2 Role {}", i),
                permissions: Permissions::try_from(0x2).unwrap(),
            };
            repo.create(input).await.unwrap();
        }

        // List roles for server 1
        let pagination = GetPaginated { page: 1, limit: 10 };
        let (roles1, total1) = repo.list_by_server(&pagination, server1_id).await.unwrap();
        assert_eq!(roles1.len(), 3);
        assert_eq!(total1, 3);
        assert!(roles1.iter().all(|r| *r.server_id == server1_id));

        // List roles for server 2
        let (roles2, total2) = repo.list_by_server(&pagination, server2_id).await.unwrap();
        assert_eq!(roles2.len(), 2);
        assert_eq!(total2, 2);
        assert!(roles2.iter().all(|r| *r.server_id == server2_id));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_role_updates_fields(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "Original Name".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role = repo.create(input).await.unwrap();

        // Update the role
        let update_input = UpdateRoleRepoInput {
            id: role.id,
            name: Some("Updated Name".to_string()),
            permissions: Some(Permissions::try_from(0x3).unwrap()),
        };
        let updated = repo.update(update_input).await.unwrap();

        assert_eq!(updated.id, role.id);
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.permissions.0, 0x3);
        assert!(updated.updated_at.is_some());

        // Verify changes persisted
        let fetched = repo.find_by_id(&role.id).await.unwrap();
        assert_eq!(fetched.name, "Updated Name");
        assert_eq!(fetched.permissions.0, 0x3);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_role_writes_outbox(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test.exchange");
        let delete_router = MessageRoutingInfo::new("test");
        let repo = PostgresRoleRepository::new(
            pool.clone(),
            create_router,
            update_router.clone(),
            delete_router,
        );
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "Original Name".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role = repo.create(input).await.unwrap();

        // Update the role
        let update_input = UpdateRoleRepoInput {
            id: role.id,
            name: Some("Updated Name".to_string()),
            permissions: Some(Permissions::try_from(0x3).unwrap()),
        };
        repo.update(update_input).await.unwrap();

        // Assert: an outbox message was written with expected routing and payload
        let row = sqlx::query(
            r#"
            SELECT exchange_name, payload
            FROM outbox_messages
            WHERE exchange_name = $1 
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(update_router.exchange_name())
        .fetch_one(&pool)
        .await
        .unwrap();

        let exchange_name: String = row.try_get("exchange_name").unwrap();
        assert_eq!(exchange_name, update_router.exchange_name());

        // Validate the payload JSON contains the update data
        let payload: serde_json::Value = row.try_get("payload").unwrap();
        assert_eq!(
            payload.get("id").and_then(|v| v.as_str()),
            Some(role.id.to_string().as_str())
        );
        assert_eq!(
            payload.get("name").and_then(|v| v.as_str()),
            Some("Updated Name")
        );
        assert_eq!(
            payload.get("permissions").and_then(|v| v.as_i64()),
            Some(0x3)
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_nonexistent_role_returns_error(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo = PostgresRoleRepository::new(pool, create_router, update_router, delete_router);
        let nonexistent_id = RoleId(Uuid::new_v4());

        let update_input = UpdateRoleRepoInput {
            id: nonexistent_id,
            name: Some("New Name".to_string()),
            permissions: None,
        };

        let result = repo.update(update_input).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected RoleNotFound error"),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_role_removes_row(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "To Delete".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role = repo.create(input).await.unwrap();

        // Delete the role
        repo.delete(&role.id).await.unwrap();

        // Verify it's gone
        let result = repo.find_by_id(&role.id).await;
        assert!(result.is_err());
        match result {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, role.id),
            _ => panic!("Expected RoleNotFound error"),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_role_writes_outbox(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test.exchange");
        let repo = PostgresRoleRepository::new(
            pool.clone(),
            create_router,
            update_router,
            delete_router.clone(),
        );
        let server_id = create_test_server(&pool, "Test Server").await;

        let input = CreateRoleInput {
            server_id,
            name: "To Delete".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role = repo.create(input).await.unwrap();
        let role_id = role.id;

        // Delete the role
        repo.delete(&role_id).await.unwrap();

        // Assert: an outbox message was written with expected routing and payload
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
        .unwrap();

        let exchange_name: String = row.try_get("exchange_name").unwrap();
        assert_eq!(exchange_name, delete_router.exchange_name());

        dbg!(&row);
        // Validate the payload JSON contains the role id
        let payload: DeleteRole = serde_json::from_value(row.try_get("payload").unwrap()).unwrap();
        // The payload is the RoleId (Uuid) serialized directly as a string
        assert_eq!(payload.role_id, role_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_nonexistent_returns_error(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo = PostgresRoleRepository::new(pool, create_router, update_router, delete_router);
        let nonexistent_id = RoleId(Uuid::new_v4());

        let result = repo.delete(&nonexistent_id).await;

        assert!(result.is_err());
        match result {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, nonexistent_id),
            _ => panic!("Expected RoleNotFound error"),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_cascade_delete_when_server_deleted(pool: PgPool) {
        let create_router = MessageRoutingInfo::new("test");
        let update_router = MessageRoutingInfo::new("test");
        let delete_router = MessageRoutingInfo::new("test");
        let repo =
            PostgresRoleRepository::new(pool.clone(), create_router, update_router, delete_router);
        let server_id = create_test_server(&pool, "Test Server").await;

        // Create roles for the server
        let input1 = CreateRoleInput {
            server_id,
            name: "Role 1".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role1 = repo.create(input1).await.unwrap();

        let input2 = CreateRoleInput {
            server_id,
            name: "Role 2".to_string(),
            permissions: Permissions::try_from(0x2).unwrap(),
        };
        let role2 = repo.create(input2).await.unwrap();

        // Delete the server
        sqlx::query("DELETE FROM servers WHERE id = $1")
            .bind(server_id)
            .execute(&pool)
            .await
            .unwrap();

        // Verify roles are also deleted (CASCADE)
        let result1 = repo.find_by_id(&role1.id).await;
        assert!(result1.is_err());
        match result1 {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, role1.id),
            _ => panic!("Expected RoleNotFound error for role1"),
        }

        let result2 = repo.find_by_id(&role2.id).await;
        assert!(result2.is_err());
        match result2 {
            Err(CoreError::RoleNotFound { id }) => assert_eq!(id, role2.id),
            _ => panic!("Expected RoleNotFound error for role2"),
        }
    }
}
