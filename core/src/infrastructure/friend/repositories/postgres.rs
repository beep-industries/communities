use sqlx::{PgPool, query_as};

use crate::domain::{
    common::{CoreError, GetPaginated},
    friend::{
        entities::{
            AcceptFriendRequestInput, CreateFriendRequestInput, DeclineFriendRequestInput,
            DeleteFriendInput, DeleteFriendRequestInput, Friend, FriendRequest, UserId,
        },
        ports::FriendshipRepository,
    },
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
    ) -> Result<(Vec<Friend>, u64), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friends WHERE user_id_1 = $1 OR user_id_2 = $1",
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CoreError::FailedToListFriends { id: *user_id })?;

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
        .map_err(|_| CoreError::FailedToListFriends { id: *user_id })?;

        Ok((friends, total_count as u64))
    }

    async fn remove_friend(&self, input: DeleteFriendInput) -> Result<(), CoreError> {
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
        .map_err(|_| CoreError::FailedToRemoveFriendship {
            user1: input.user_id_1,
            user2: input.user_id_2,
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::FailedToRemoveFriendship {
                user1: input.user_id_1,
                user2: input.user_id_2,
            });
        }
        Ok(())
    }

    async fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, u64), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;

        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friend_requests WHERE user_id_requested = $1",
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CoreError::FriendNotFound {
            id: user_id.clone(),
        })?;

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
        .map_err(|_| CoreError::FriendNotFound {
            id: user_id.clone(),
        })?;

        Ok((friend_requests, total_count as u64))
    }

    async fn create_request(
        &self,
        input: CreateFriendRequestInput,
    ) -> Result<FriendRequest, CoreError> {
        query_as!(
            FriendRequest,
            r#"
            INSERT INTO friend_requests (user_id_requested, user_id_invited, status)
            VALUES ($1, $2, $3)
            RETURNING user_id_requested, user_id_invited, status, created_at
            "#,
            input.user_id_requested.0,
            input.user_id_invited.0,
            0 // by default 0 means pending
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CoreError::FailedToCreateFriendship {
            user1: input.user_id_invited,
            user2: input.user_id_requested,
        })
    }

    async fn accept_request(&self, input: AcceptFriendRequestInput) -> Result<Friend, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::UnkownError {
                message: e.to_string(),
            })?;

        let delete_result = sqlx::query!(
            r#"
            DELETE FROM friend_requests
            WHERE user_id_requested = $1 AND user_id_invited = $2 AND status = 0
            "#,
            input.user_id_requested.0,
            input.user_id_invited.0
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToRemoveFriendship {
            user1: input.user_id_invited,
            user2: input.user_id_requested,
        })?;

        if delete_result.rows_affected() == 0 {
            // if no rows were affected, the friend request did not exist and the operation fails
            return Err(CoreError::FailedToRemoveFriendship {
                user1: input.user_id_invited,
                user2: input.user_id_requested,
            });
        }

        let friend = query_as!(
            Friend,
            r#"
            INSERT INTO friends (user_id_1, user_id_2)
            VALUES ($1, $2)
            RETURNING user_id_1, user_id_2, created_at
            "#,
            input.user_id_requested.clone().0,
            input.user_id_invited.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToCreateFriendship {
            user1: input.user_id_invited,
            user2: input.user_id_requested,
        })?;

        tx.commit()
            .await
            .map_err(|_| CoreError::FailedToCreateFriendship {
                user1: input.user_id_invited,
                user2: input.user_id_requested,
            })?;

        Ok(friend)
    }

    async fn decline_request(
        &self,
        input: DeclineFriendRequestInput,
    ) -> Result<FriendRequest, CoreError> {
        sqlx::query_as!(
            FriendRequest,
            r#"
            UPDATE friend_requests
            SET status = $3
            WHERE user_id_requested = $1 AND user_id_invited = $2
            RETURNING user_id_requested, user_id_invited, status, created_at
            "#,
            input.user_id_requested.0,
            input.user_id_invited.0,
            1 // 1 means declined
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CoreError::FailedToCreateFriendship {
            user1: input.user_id_invited,
            user2: input.user_id_requested,
        })
    }

    async fn remove_request(&self, input: DeleteFriendRequestInput) -> Result<(), CoreError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friend_requests
            WHERE user_id_requested = $1 AND user_id_invited = $2
            "#,
            input.user_id_requested.0,
            input.user_id_invited.0
        )
        .execute(&self.pool)
        .await
        .map_err(|_| CoreError::FailedToRemoveFriendship {
            user1: input.user_id_invited,
            user2: input.user_id_requested,
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::FailedToRemoveFriendship {
                user1: input.user_id_invited,
                user2: input.user_id_requested,
            });
        }
        Ok(())
    }
}
