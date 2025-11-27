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
