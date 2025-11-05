use std::sync::Arc;

use crate::{
    database::Database,
    models::server::{CreateServer, Server},
};

pub struct ServerRepository {
    database: Arc<Database>,
}

impl ServerRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub async fn create(&self, server_info: CreateServer) -> Result<Server, sqlx::Error> {
        let server = sqlx::query_as::<_, Server>(
            r#"
            INSERT INTO servers (name, banner_url, picture_url, description, owner_id)
            VALUES ($1, $2, $3, $4, $5::uuid)
            RETURNING id::text, name, banner_url, picture_url, description, owner_id::text, created_at::text, updated_at::text
            "#
        )
        .bind(&server_info.name)
        .bind(&server_info.banner_url)
        .bind(&server_info.picture_url)
        .bind(&server_info.description)
        .bind(&server_info.owner_id)
        .fetch_one(&self.database.pool)
        .await?;

        Ok(server)
    }

    pub async fn get_by_id(&self, id: String) -> Result<Option<Server>, sqlx::Error> {
        let server = sqlx::query_as::<_, Server>(
            r#"
            SELECT id::text, name, banner_url, picture_url, description, owner_id::text, created_at::text, updated_at::text
            FROM servers 
            WHERE id = $1::uuid
            "#
        )
        .bind(id)
        .fetch_optional(&self.database.pool)
        .await?;

        Ok(server)
    }
}
