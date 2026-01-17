use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        friend::entities::UserId,
        server::entities::ServerId,
        server_member::{
            MemberId,
            entities::{CreateMemberInput, DeleteMemberEvent, ServerMember, UpdateMemberInput},
            ports::MemberRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

#[derive(Clone)]
pub struct PostgresMemberRepository {
    pub(crate) pool: PgPool,
    user_join_server_router: MessageRoutingInfo,
    delete_member_router: MessageRoutingInfo,
}

impl PostgresMemberRepository {
    pub fn new(
        pool: PgPool,
        delete_member_router: MessageRoutingInfo,
        user_join_server_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            delete_member_router,
            user_join_server_router,
        }
    }
}

impl MemberRepository for PostgresMemberRepository {
    async fn insert(&self, input: CreateMemberInput) -> Result<ServerMember, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;
        let member_id = Uuid::new_v4();

        // Insert the member into the database
        let server_member = sqlx::query_as!(
            ServerMember,
            r#"
            INSERT INTO server_members (id, server_id, user_id, nickname)
            VALUES ($1, $2, $3, $4)
            RETURNING id, server_id, user_id, nickname, joined_at, updated_at
            "#,
            member_id,
            *input.server_id,
            *input.user_id,
            input.nickname,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to insert member: {}", e),
        })?;

        let member_join_server =
            OutboxEventRecord::new(self.user_join_server_router.clone(), server_member.clone());

        member_join_server.write(&mut *tx).await?;

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(server_member)
    }

    async fn find_by_server_and_user(
        &self,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> Result<ServerMember, CoreError> {
        dbg!(**server_id);
        dbg!(**user_id);
        let row = sqlx::query_as!(
            ServerMember,
            r#"
            SELECT id, server_id, user_id, nickname, joined_at, updated_at
            FROM server_members
            WHERE server_id = $1 AND user_id = $2
            "#,
            **server_id,
            **user_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to find member: {}", e),
        })?;
        dbg!("Finding members failed");
        match row {
            Some(member) => Ok(member),
            None => Err(CoreError::MemberNotFound {
                server_id: *server_id,
                user_id: *user_id,
            }),
        }
    }

    async fn list_by_server(
        &self,
        server_id: &ServerId,
        pagination: &GetPaginated,
    ) -> Result<(Vec<ServerMember>, TotalPaginatedElements), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;
        let limit = std::cmp::min(pagination.limit, 50) as i64;

        // Get total count of members for this server
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM server_members WHERE server_id = $1")
                .bind(server_id.0)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to count members: {}", e),
                })?;

        // Get paginated members
        let rows = sqlx::query(
            r#"
            SELECT id, server_id, user_id, nickname, joined_at, updated_at
            FROM server_members
            WHERE server_id = $1
            ORDER BY joined_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(server_id.0)
        .bind(limit)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to list members: {}", e),
        })?;

        let members: Vec<ServerMember> = rows.into_iter().map(|r| (&r).into()).collect();

        Ok((members, total as u64))
    }

    async fn update(&self, input: UpdateMemberInput) -> Result<ServerMember, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;

        // Update the member and return the updated row
        let row = sqlx::query(
            r#"
            UPDATE server_members
            SET nickname = $1, updated_at = NOW()
            WHERE server_id = $2 AND user_id = $3
            RETURNING id, server_id, user_id, nickname, joined_at, updated_at
            "#,
        )
        .bind(&input.nickname)
        .bind(input.server_id.0)
        .bind(input.user_id.0)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to update member: {}", e),
        })?;

        let row = row.ok_or_else(|| CoreError::MemberNotFound {
            server_id: input.server_id,
            user_id: input.user_id,
        })?;

        let member: ServerMember = (&row).into();

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(member)
    }

    async fn delete(&self, server_id: &ServerId, user_id: &UserId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to begin transaction: {}", e),
            })?;

        // Delete the member
        let result =
            sqlx::query(r#"DELETE FROM server_members WHERE server_id = $1 AND user_id = $2"#)
                .bind(server_id.0)
                .bind(user_id.0)
                .execute(&mut *tx)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to delete member: {}", e),
                })?;

        if result.rows_affected() == 0 {
            dbg!("Error on deletje");
            return Err(CoreError::MemberNotFound {
                server_id: *server_id,
                user_id: *user_id,
            });
        }

        // Write the delete event to the outbox table
        let delete_event = DeleteMemberEvent {
            server_id: *server_id,
            user_id: *user_id,
        };
        let outbox_event = OutboxEventRecord::new(self.delete_member_router.clone(), delete_event);
        outbox_event.write(&mut *tx).await?;

        tx.commit().await.map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(())
    }

    async fn find_by_id(&self, member_id: MemberId) -> Result<ServerMember, CoreError> {
        let row = sqlx::query_as!(
            ServerMember,
            r#"
            SELECT id, server_id, user_id, nickname, joined_at, updated_at
            FROM server_members
            WHERE id = $1 
            "#,
            *member_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to find member: {}", e),
        })?;
        match row {
            Some(member) => Ok(member),
            None => Err(CoreError::MemberNotFoundById { member_id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::friend::entities::UserId;
    use crate::domain::server::entities::{ServerId, ServerVisibility};
    use crate::infrastructure::outbox::MessageRouter;
    use sqlx::Row;

    // Helper function to create a test server
    async fn create_test_server(pool: &PgPool, server_id: ServerId) -> Result<(), CoreError> {
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
    async fn test_insert_member_writes_row(pool: PgPool) -> Result<(), CoreError> {
        let delete_router = MessageRoutingInfo::new("member.exchange");

        let repository = PostgresMemberRepository::new(
            pool.clone(),
            delete_router,
            MessageRoutingInfo::default(),
        );

        let server_id = ServerId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Create a test server first
        create_test_server(&pool, server_id).await?;

        let input = CreateMemberInput {
            server_id,
            user_id,
            nickname: Some("TestNick".to_string()),
        };

        // Act: insert member
        let created = repository.insert(input.clone()).await?;

        // Assert: returned fields
        assert_eq!(created.server_id, server_id);
        assert_eq!(created.user_id, user_id);
        assert_eq!(created.nickname, Some("TestNick".to_string()));
        assert!(created.updated_at.is_none());

        // Assert: it can be fetched back
        let fetched = repository
            .find_by_server_and_user(&server_id, &user_id)
            .await?;
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.server_id, created.server_id);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_server_and_user_returns_member(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        let server_id = ServerId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Create a test server first
        create_test_server(&pool, server_id).await?;

        let input = CreateMemberInput {
            server_id,
            user_id,
            nickname: Some("FindMe".to_string()),
        };

        // Arrange: insert a member
        let created = repository.insert(input).await?;

        // Act: find the member
        let found = repository
            .find_by_server_and_user(&server_id, &user_id)
            .await?;

        // Assert: member is found
        assert_eq!(found.id, created.id);
        assert_eq!(found.server_id, server_id);
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.nickname, Some("FindMe".to_string()));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_server_and_user_returns_error_for_nonexistent(
        pool: PgPool,
    ) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        // Try to find a member that doesn't exist
        let nonexistent_server = ServerId(Uuid::new_v4());
        let nonexistent_user = UserId(Uuid::new_v4());
        let result = repository
            .find_by_server_and_user(&nonexistent_server, &nonexistent_user)
            .await;
        assert!(matches!(result, Err(CoreError::MemberNotFound { .. })));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_by_server_returns_paginated_members(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        let server_id = ServerId(Uuid::new_v4());

        // Create a test server first
        create_test_server(&pool, server_id).await?;

        // Arrange: insert multiple members for the same server
        for i in 0..5 {
            let input = CreateMemberInput {
                server_id,
                user_id: UserId(Uuid::new_v4()),
                nickname: Some(format!("Member{}", i)),
            };
            repository.insert(input).await?;
        }

        // Act: list members with pagination
        let pagination = GetPaginated { page: 1, limit: 3 };
        let (members, total) = repository.list_by_server(&server_id, &pagination).await?;

        // Assert: correct count and total
        assert_eq!(members.len(), 3);
        assert_eq!(total, 5);

        // Assert: members are ordered by joined_at DESC (newest first)
        for i in 0..members.len() - 1 {
            assert!(members[i].joined_at >= members[i + 1].joined_at);
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_member_updates_fields(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        let server_id = ServerId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Create a test server first
        create_test_server(&pool, server_id).await?;

        // Arrange: insert a member first
        let input = CreateMemberInput {
            server_id,
            user_id,
            nickname: Some("OldNick".to_string()),
        };
        repository.insert(input).await?;

        // Act: update the member
        let update_input = UpdateMemberInput {
            server_id,
            user_id,
            nickname: Some("NewNick".to_string()),
        };
        let updated = repository.update(update_input.clone()).await?;

        // Assert: returned member has updated fields
        assert_eq!(updated.server_id, server_id);
        assert_eq!(updated.user_id, user_id);
        assert_eq!(updated.nickname, Some("NewNick".to_string()));
        assert!(updated.updated_at.is_some());

        // Assert: update persisted in database
        let fetched = repository
            .find_by_server_and_user(&server_id, &user_id)
            .await?;
        assert_eq!(fetched.nickname, Some("NewNick".to_string()));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_nonexistent_member_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        // Try to update a member that doesn't exist
        let nonexistent_server = ServerId(Uuid::new_v4());
        let nonexistent_user = UserId(Uuid::new_v4());
        let update_input = UpdateMemberInput {
            server_id: nonexistent_server,
            user_id: nonexistent_user,
            nickname: Some("NewNick".to_string()),
        };

        let result = repository.update(update_input).await;

        // Assert: should return MemberNotFound error
        assert!(result.is_err());
        match result {
            Err(CoreError::MemberNotFound { server_id, user_id }) => {
                assert_eq!(server_id, nonexistent_server);
                assert_eq!(user_id, nonexistent_user);
            }
            _ => panic!("Expected MemberNotFound error"),
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_member_removes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
        let delete_router = MessageRoutingInfo::new("member.exchange");
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            delete_router.clone(),
            MessageRoutingInfo::default(),
        );

        let server_id = ServerId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Create a test server first
        create_test_server(&pool, server_id).await?;

        // Arrange: insert a member first
        let input = CreateMemberInput {
            server_id,
            user_id,
            nickname: Some("ToDelete".to_string()),
        };
        repository.insert(input).await?;

        // Act: delete the member
        repository.delete(&server_id, &user_id).await?;

        // Assert: member is gone
        let fetched = repository
            .find_by_server_and_user(&server_id, &user_id)
            .await;

        assert!(matches!(fetched, Err(CoreError::MemberNotFound { .. })));
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

        let _exchange_name: String = row
            .try_get("exchange_name")
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
        let payload: serde_json::Value = row
            .try_get("payload")
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let server_str = server_id.0.to_string();
        let user_str = user_id.0.to_string();
        assert_eq!(
            payload.get("server_id").and_then(|v| v.as_str()),
            Some(server_str.as_str())
        );
        assert_eq!(
            payload.get("user_id").and_then(|v| v.as_str()),
            Some(user_str.as_str())
        );

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_nonexistent_member_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresMemberRepository::new(
            pool.clone(),
            MessageRoutingInfo::default(),
            MessageRoutingInfo::default(),
        );

        // Try to delete a member that doesn't exist
        let nonexistent_server = ServerId(Uuid::new_v4());
        let nonexistent_user = UserId(Uuid::new_v4());
        let result = repository
            .delete(&nonexistent_server, &nonexistent_user)
            .await;

        // Assert: should return MemberNotFound error
        assert!(result.is_err());
        match result {
            Err(CoreError::MemberNotFound { server_id, user_id }) => {
                assert_eq!(server_id, nonexistent_server);
                assert_eq!(user_id, nonexistent_user);
            }
            _ => panic!("Expected MemberNotFound error"),
        }

        Ok(())
    }
}
