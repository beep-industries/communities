use sqlx::{PgPool, postgres::PgListener};
use uuid::Uuid;

#[cfg(test)]
use futures_util::StreamExt;

use crate::domain::{
    common::{GetPaginated, TotalPaginatedElements},
    outbox::{
        entities::{OutboxMessage, OutboxMessageStream, OutboxStatus},
        error::OutboxError,
        ports::OutboxRepository,
    },
};

/// PostgreSQL implementation of the outbox repository
#[derive(Clone)]
pub struct PostgresOutboxRepository {
    pool: PgPool,
}

impl PostgresOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OutboxRepository for PostgresOutboxRepository {
    /// Retrieve outbox events with pagination support
    async fn get(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<OutboxMessage>, TotalPaginatedElements), OutboxError> {
        let offset = (pagination.page - 1) * pagination.limit;

        // Get total count of READY messages only
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_messages WHERE status = 'READY'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| OutboxError::DatabaseError)?;

        // Fetch paginated results - only READY messages
        let rows = sqlx::query!(
            r#"
            SELECT id, exchange_name, payload, status, failed_at, created_at
            FROM outbox_messages
            WHERE status = 'READY'
            ORDER BY created_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            pagination.limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| OutboxError::DatabaseError)?;

        let messages = rows
            .into_iter()
            .map(|row| {
                let status = match row.status.as_str() {
                    "READY" => OutboxStatus::Ready,
                    "SENT" => OutboxStatus::Sent,
                    _ => OutboxStatus::Ready, // default fallback
                };

                OutboxMessage {
                    id: row.id,
                    exchange_name: row.exchange_name,
                    payload: row.payload,
                    status,
                    failed_at: row.failed_at,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok((messages, total_count as TotalPaginatedElements))
    }

    /// Listen to real-time outbox event notifications using PostgreSQL LISTEN/NOTIFY
    async fn listen_outbox_event(&self) -> Result<OutboxMessageStream, OutboxError> {
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .map_err(|e| OutboxError::ListenerError { msg: e.to_string() })?;

        listener
            .listen("outbox_channel")
            .await
            .map_err(|e| OutboxError::ListenerError { msg: e.to_string() })?;

        let outbox_event_stream = OutboxMessageStream::from(listener);
        Ok(outbox_event_stream)
    }

    /// Delete all outbox events that have been marked as sent
    async fn delete_marked(&self) -> Result<u64, OutboxError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM outbox_messages
            WHERE status = 'SENT'
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|_| OutboxError::DatabaseError)?;

        Ok(result.rows_affected())
    }

    /// Update the status of a specific outbox event
    async fn mark_event(
        &self,
        id: Uuid,
        status: OutboxStatus,
    ) -> Result<OutboxMessage, OutboxError> {
        let status_str = status.as_str();

        let row = sqlx::query!(
            r#"
            UPDATE outbox_messages
            SET status = $2
            WHERE id = $1
            RETURNING id, exchange_name, payload, status, failed_at, created_at
            "#,
            id,
            status_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| OutboxError::DatabaseError)?;

        match row {
            Some(row) => {
                let status = match row.status.as_str() {
                    "READY" => OutboxStatus::Ready,
                    "SENT" => OutboxStatus::Sent,
                    _ => OutboxStatus::Ready,
                };

                Ok(OutboxMessage {
                    id: row.id,
                    exchange_name: row.exchange_name,
                    payload: row.payload,
                    status,
                    failed_at: row.failed_at,
                    created_at: row.created_at,
                })
            }
            None => Err(OutboxError::EventNotFound { id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::CoreError;

    // Helper function to insert a test outbox message
    async fn insert_test_message(
        pool: &PgPool,
        exchange: &str,
        status: &str,
    ) -> Result<Uuid, CoreError> {
        let id = Uuid::new_v4();
        let payload = serde_json::json!({
            "test": "data"
        });

        sqlx::query!(
            r#"
            INSERT INTO outbox_messages (id, exchange_name, payload, status)
            VALUES ($1, $2, $3, $4)
            "#,
            id,
            exchange,
            payload,
            status
        )
        .execute(pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to insert test message: {}", e),
        })?;

        Ok(id)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_outbox_events_paginated(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        // Insert multiple READY test messages
        for _ in 0..5 {
            insert_test_message(&pool, "test.exchange", "READY")
                .await?;
        }

        // Insert some SENT messages that should NOT be returned
        insert_test_message(&pool, "test.exchange", "SENT").await?;
        insert_test_message(&pool, "test.exchange", "SENT").await?;

        // Get first page
        let pagination = GetPaginated { page: 1, limit: 3 };
        let (messages, total) =
            repository
                .get(&pagination)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to get outbox events: {:?}", e),
                })?;

        assert_eq!(messages.len(), 3);
        assert_eq!(total, 5); // Should only count READY messages
        // Verify all returned messages are READY
        for msg in &messages {
            assert_eq!(msg.status, OutboxStatus::Ready);
        }

        // Get second page
        let pagination = GetPaginated { page: 2, limit: 3 };
        let (messages, total) =
            repository
                .get(&pagination)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to get outbox events: {:?}", e),
                })?;

        assert_eq!(messages.len(), 2);
        assert_eq!(total, 5); // Should only count READY messages
        // Verify all returned messages are READY
        for msg in &messages {
            assert_eq!(msg.status, OutboxStatus::Ready);
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_empty_outbox(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        let pagination = GetPaginated { page: 1, limit: 10 };
        let (messages, total) =
            repository
                .get(&pagination)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to get outbox events: {:?}", e),
                })?;

        assert_eq!(messages.len(), 0);
        assert_eq!(total, 0);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_listen_outbox_event_receives_notifications(
        pool: PgPool,
    ) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        // Start listening and get the stream
        let mut stream =
            repository
                .listen_outbox_event()
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to create listener: {:?}", e),
                })?;

        // Insert a message in a separate task (triggers notification)
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let _ = insert_test_message(&pool_clone, "test.exchange", "READY").await;
        });

        // Wait for notification with timeout
        let outbox_message =
            tokio::time::timeout(tokio::time::Duration::from_secs(5), stream.next())
                .await
                .map_err(|_| CoreError::DatabaseError {
                    msg: "Timeout waiting for notification".to_string(),
                })?
                .ok_or_else(|| CoreError::DatabaseError {
                    msg: "Stream ended without notification".to_string(),
                })?
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to receive notification: {:?}", e),
                })?;

        // Verify the received message has the expected properties
        assert_eq!(outbox_message.exchange_name, "test.exchange");
        assert_eq!(outbox_message.status, OutboxStatus::Ready);
        assert_eq!(outbox_message.payload["test"], "data");

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_mark_event_updates_status(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        let id = insert_test_message(&pool, "test.exchange", "READY").await?;

        // Mark as sent
        let updated = repository
            .mark_event(id, OutboxStatus::Sent)
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to mark event: {:?}", e),
            })?;

        assert_eq!(updated.id, id);
        assert_eq!(updated.status, OutboxStatus::Sent);

        // Verify in database
        let row = sqlx::query!(
            r#"
            SELECT status FROM outbox_messages WHERE id = $1
            "#,
            id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to query database: {}", e),
        })?;

        assert_eq!(row.status, "SENT");

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_mark_nonexistent_event_returns_error(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        let nonexistent_id = Uuid::new_v4();
        let result = repository
            .mark_event(nonexistent_id, OutboxStatus::Sent)
            .await;

        assert!(result.is_err());
        match result {
            Err(OutboxError::EventNotFound { id }) => {
                assert_eq!(id, nonexistent_id);
            }
            _ => panic!("Expected EventNotFound error"),
        }

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_marked_removes_sent_events(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        // Insert SENT messages
        insert_test_message(&pool, "test.exchange", "SENT").await?;
        insert_test_message(&pool, "test.exchange", "SENT").await?;

        // Delete marked events
        let deleted = repository
            .delete_marked()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to delete marked events: {:?}", e),
            })?;

        assert_eq!(deleted, 2);

        // Verify deletion
        let count: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM outbox_messages WHERE status = 'SENT'"#)
                .fetch_one(&pool)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to count messages: {}", e),
                })?;

        assert_eq!(count, 0);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_marked_preserves_ready_events(pool: PgPool) -> Result<(), CoreError> {
        let repository = PostgresOutboxRepository::new(pool.clone());

        // Insert READY and SENT messages
        insert_test_message(&pool, "test.exchange", "READY").await?;
        insert_test_message(&pool, "test.exchange", "READY").await?;
        insert_test_message(&pool, "test.exchange", "SENT").await?;

        // Delete marked events
        let deleted = repository
            .delete_marked()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to delete marked events: {:?}", e),
            })?;

        assert_eq!(deleted, 1);

        // Verify READY events still exist
        let ready_count: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM outbox_messages WHERE status = 'READY'"#)
                .fetch_one(&pool)
                .await
                .map_err(|e| CoreError::DatabaseError {
                    msg: format!("Failed to count READY messages: {}", e),
                })?;

        assert_eq!(ready_count, 2);

        Ok(())
    }
}
