use sqlx::{PgPool, query_as};

use crate::{
    domain::{
        common::{GetPaginated, TotalPaginatedElements},
        friend::{
            entities::{DeleteFriendInput, Friend, FriendRequest, UserId},
            ports::FriendshipRepository,
        },
    },
    infrastructure::friend::repositories::error::FriendshipError,
};

#[derive(Clone)]
pub struct PostgresFriendshipRepository {
    pool: PgPool,
}

impl PostgresFriendshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FriendshipRepository for PostgresFriendshipRepository {
    async fn list_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<Friend>, TotalPaginatedElements), FriendshipError> {
        let offset = (pagination.page - 1) * pagination.limit;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friends WHERE user_id_1 = $1 OR user_id_2 = $1",
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        let friends = query_as!(
            Friend,
            r#"
            SELECT user_id_1, user_id_2, created_at
            FROM friends
            WHERE user_id_1 = $1 OR user_id_2 = $1
            ORDER BY created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
            user_id.0,
            (pagination.limit as i64),
            (offset as i64)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        Ok((friends, total_count as TotalPaginatedElements))
    }

    async fn get_friend(
        &self,
        user_id_1: &UserId,
        user_id_2: &UserId,
    ) -> Result<Option<Friend>, FriendshipError> {
        let friend = query_as!(
            Friend,
            r#"
            SELECT user_id_1, user_id_2, created_at
            FROM Friends
            WHERE user_id_1 = $1 AND user_id_2 = $2 OR user_id_1 = $2 AND user_id_2 = $1
            "#,
            user_id_1.0,
            user_id_2.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        Ok(friend)
    }

    async fn remove_friend(&self, input: DeleteFriendInput) -> Result<(), FriendshipError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friends
            WHERE (user_id_1 = $1 AND user_id_2 = $2) OR (user_id_1 = $2 AND user_id_2 = $1)
            "#,
            input.user_id_1.0,
            input.user_id_2.0
        )
        .execute(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(FriendshipError::FriendshipNotFound);
        }

        Ok(())
    }

    async fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        let offset = (pagination.page - 1) * pagination.limit;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friend_requests WHERE user_id_requested = $1",
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        let friend_requests = query_as!(
            FriendRequest,
            r#"
            SELECT user_id_requested, user_id_invited, status, created_at
            FROM friend_requests
            WHERE user_id_requested = $1
            ORDER BY status ASC, created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
            user_id.0,
            (pagination.limit as i64),
            (offset as i64)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        Ok((friend_requests, total_count as TotalPaginatedElements))
    }

    async fn list_invitations(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        let offset = (pagination.page - 1) * pagination.limit;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friend_requests WHERE user_id_invited = $1 AND status = 0",
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        let friend_requests = query_as!(
            FriendRequest,
            r#"
            SELECT user_id_requested, user_id_invited, status, created_at
            FROM friend_requests
            WHERE user_id_invited = $1
            AND status = 0
            ORDER BY status ASC, created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
            user_id.0,
            (pagination.limit as i64),
            (offset as i64)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        Ok((friend_requests, total_count as TotalPaginatedElements))
    }

    async fn get_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<Option<FriendRequest>, FriendshipError> {
        let request = query_as!(
            FriendRequest,
            r#"
            SELECT user_id_requested, user_id_invited, status, created_at
            FROM friend_requests
            WHERE user_id_requested = $1 AND user_id_invited = $2
            "#,
            user_id_requested.0,
            user_id_invited.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        Ok(request)
    }

    async fn create_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        query_as!(
            FriendRequest,
            r#"
            INSERT INTO friend_requests (user_id_requested, user_id_invited, status)
            VALUES ($1, $2, $3)
            RETURNING user_id_requested, user_id_invited, status, created_at
            "#,
            user_id_requested.0,
            user_id_invited.0,
            0 // by default 0 means pending
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| FriendshipError::FriendRequestAlreadyExists)
    }

    async fn accept_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<Friend, FriendshipError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| FriendshipError::DatabaseError)?;

        let delete_result = sqlx::query!(
            r#"
            DELETE FROM friend_requests
            WHERE user_id_requested = $1 AND user_id_invited = $2 AND status = 0
            "#,
            user_id_requested.0,
            user_id_invited.0
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        if delete_result.rows_affected() == 0 {
            // if no rows were affected, the friend request did not exist and the operation fails
            return Err(FriendshipError::FriendRequestNotFound);
        }

        let friend = query_as!(
            Friend,
            r#"
            INSERT INTO friends (user_id_1, user_id_2)
            VALUES ($1, $2)
            RETURNING user_id_1, user_id_2, created_at
            "#,
            user_id_requested.0,
            user_id_invited.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| FriendshipError::FriendshipAlreadyExists)?;

        tx.commit()
            .await
            .map_err(|_| FriendshipError::DatabaseError)?;

        Ok(friend)
    }

    async fn decline_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        sqlx::query_as!(
            FriendRequest,
            r#"
            UPDATE friend_requests
            SET status = $3
            WHERE user_id_requested = $1 AND user_id_invited = $2
            RETURNING user_id_requested, user_id_invited, status, created_at
            "#,
            user_id_requested.0,
            user_id_invited.0,
            1 // 1 means declined
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| FriendshipError::FriendRequestNotFound)
    }

    async fn remove_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<(), FriendshipError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friend_requests
            WHERE user_id_requested = $1 AND user_id_invited = $2
            "#,
            user_id_requested.0,
            user_id_invited.0
        )
        .execute(&self.pool)
        .await
        .map_err(|_| FriendshipError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(FriendshipError::FriendRequestNotFound);
        }

        Ok(())
    }
}
