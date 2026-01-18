use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    common::CoreError,
    friend::entities::UserId,
    server_invitation::{
        entities::{
            InsertServerInvitationInput, ServerInvitation, ServerInvitationId,
            UpdateServerInvitationInput,
        },
        ports::ServerInvitationRepository,
    },
};

#[derive(Clone)]
pub struct PostgresServerInvitationRepository {
    pub(crate) pool: PgPool,
}

impl PostgresServerInvitationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ServerInvitationRepository for PostgresServerInvitationRepository {
    async fn insert(
        &self,
        input: InsertServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO server_invitations (server_id, inviter_id, invitee_id, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, server_id, inviter_id, invitee_id, 
                      status as "status: server_invitation_status", created_at, updated_at, expires_at
            "#,
            input.server_id.0,
            input.inviter_id.0,
            input.invitee_id.map(|id| id.0),
            input.expires_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to insert server invitation: {}", e),
        })?;

        Ok(ServerInvitation {
            id: ServerInvitationId(row.id),
            server_id: row.server_id.into(),
            inviter_id: UserId(row.inviter_id),
            invitee_id: row.invitee_id.map(UserId),
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at,
        })
    }

    async fn find_by_id(&self, id: &ServerInvitationId) -> Result<ServerInvitation, CoreError> {
        let row = sqlx::query!(
            r#"
            SELECT id, server_id, inviter_id, invitee_id, 
                   status as "status: server_invitation_status", created_at, updated_at, expires_at
            FROM server_invitations
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to fetch server invitation: {}", e),
        })?;

        match row {
            Some(r) => Ok(ServerInvitation {
                id: ServerInvitationId(r.id),
                server_id: r.server_id.into(),
                inviter_id: UserId(r.inviter_id),
                invitee_id: r.invitee_id.map(UserId),
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
            }),
            None => Err(CoreError::DatabaseError {
                msg: format!("Server invitation not found with id: {}", id),
            }),
        }
    }

    async fn update(
        &self,
        input: UpdateServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        let row = sqlx::query!(
            r#"
            UPDATE server_invitations
            SET status = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, server_id, inviter_id, invitee_id, 
                      status as "status: server_invitation_status", created_at, updated_at, expires_at
            "#,
            input.id.0,
            input.status as _,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to update server invitation: {}", e),
        })?;

        match row {
            Some(r) => Ok(ServerInvitation {
                id: ServerInvitationId(r.id),
                server_id: r.server_id.into(),
                inviter_id: UserId(r.inviter_id),
                invitee_id: r.invitee_id.map(UserId),
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
            }),
            None => Err(CoreError::DatabaseError {
                msg: format!("Server invitation not found with id: {}", input.id),
            }),
        }
    }

    async fn delete(&self, id: &ServerInvitationId) -> Result<(), CoreError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM server_invitations
            WHERE id = $1
            "#,
            id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError {
            msg: format!("Failed to delete server invitation: {}", e),
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::DatabaseError {
                msg: format!("Server invitation not found with id: {}", id),
            });
        }

        Ok(())
    }
}
