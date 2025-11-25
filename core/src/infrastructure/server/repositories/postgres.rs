use sqlx::{PgPool, query_as};

use crate::{domain::{
    common::CoreError,
    server::{
        entities::{InsertServerInput, Server, ServerId},
        ports::ServerRepository,
    },
}, write_outbox_event};

#[derive(Clone)]
pub struct PostgresServerRepository {
    pub(crate) pool: PgPool,
}

impl PostgresServerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        
        write_outbox_event(&mut *tx, &input).await?;
        
        tx.commit()
            .await
            .map_err(|_| CoreError::FailedToInsertServer { name: input.name })?;

        Ok(server)
    }
}
