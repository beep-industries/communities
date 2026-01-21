use std::future::Future;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::common::{CoreError, GetPaginated, TotalPaginatedElements};
use crate::domain::friend::entities::UserId;
use crate::domain::server::entities::ServerId;

use super::entities::{CreateMemberInput, MemberId, ServerMember, UpdateMemberInput};

/// Repository trait for server member persistence
pub trait MemberRepository: Send + Sync {
    /// Insert a new server member
    fn insert(
        &self,
        input: CreateMemberInput,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    /// Find a member by server ID and user ID
    fn find_by_server_and_user(
        &self,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    /// List all members of a server with pagination
    fn list_by_server(
        &self,
        server_id: &ServerId,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<ServerMember>, TotalPaginatedElements), CoreError>> + Send;

    /// Update a server member
    fn update(
        &self,
        input: UpdateMemberInput,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    /// Delete a server member
    fn delete(
        &self,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn find_by_id(
        &self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;
}

/// Service trait for server member business logic
pub trait MemberService: Send + Sync {
    /// Create a new server member
    ///
    /// # Arguments
    /// * `input` - CreateMemberInput containing server_id, user_id, optional role and nickname
    ///
    /// # Returns
    /// * `Ok(ServerMember)` - The created member
    /// * `Err(CoreError::ServerNotFound)` - If the server doesn't exist
    /// * `Err(CoreError::MemberAlreadyExists)` - If the user is already a member
    /// * `Err(CoreError::InvalidMemberNickname)` - If the nickname is empty or whitespace
    fn create_member(
        &self,
        input: CreateMemberInput,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    /// List all members of a server with pagination
    ///
    /// # Arguments
    /// * `server_id` - The server to list members from
    /// * `pagination` - Page number and limit
    ///
    /// # Returns
    /// * `Ok((Vec<ServerMember>, TotalPaginatedElements))` - List of members and total count
    /// * `Err(CoreError::ServerNotFound)` - If the server doesn't exist
    fn list_members(
        &self,
        server_id: ServerId,
        pagination: GetPaginated,
    ) -> impl Future<Output = Result<(Vec<ServerMember>, TotalPaginatedElements), CoreError>> + Send;

    /// Update a server member
    ///
    /// # Arguments
    /// * `input` - UpdateMemberInput containing server_id, user_id, and optional fields to update
    ///
    /// # Returns
    /// * `Ok(ServerMember)` - The updated member
    /// * `Err(CoreError::MemberNotFound)` - If the member doesn't exist
    /// * `Err(CoreError::InvalidMemberNickname)` - If the nickname is empty or whitespace
    fn update_member(
        &self,
        input: UpdateMemberInput,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    /// Delete a server member
    ///
    /// # Arguments
    /// * `server_id` - The server to remove the member from
    /// * `user_id` - The user to remove
    ///
    /// # Returns
    /// * `Ok(())` - Member successfully deleted
    /// * `Err(CoreError::MemberNotFound)` - If the member doesn't exist
    fn delete_member(
        &self,
        server_id: ServerId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Get a server member by server ID and user ID
    /// # Arguments
    /// * `server_id` - The server ID
    /// * `user_id` - The user ID
    /// # Returns
    /// * `Ok(ServerMember)` - The found member
    /// * `Err(CoreError::MemberNotFound)` - If the member doesn't exist    
    fn get_member(
        &self,
        server_id: ServerId,
        user_id: UserId,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;

    fn get_member_by_id(
        &self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<ServerMember, CoreError>> + Send;
}

/// Mock implementation of MemberRepository for testing
#[derive(Clone)]
pub struct MockMemberRepository {
    members: Arc<Mutex<Vec<ServerMember>>>,
}

impl MockMemberRepository {
    pub fn new() -> Self {
        Self {
            members: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MemberRepository for MockMemberRepository {
    async fn insert(&self, input: CreateMemberInput) -> Result<ServerMember, CoreError> {
        let mut members = self.members.lock().unwrap();

        let member = ServerMember {
            id: MemberId::from(Uuid::new_v4()),
            server_id: input.server_id,
            user_id: input.user_id,
            nickname: input.nickname,
            joined_at: Utc::now(),
            updated_at: None,
        };

        members.push(member.clone());
        Ok(member)
    }

    async fn find_by_server_and_user(
        &self,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> Result<ServerMember, CoreError> {
        let members = self.members.lock().unwrap();
        let member = members
            .iter()
            .find(|m| m.server_id == *server_id && m.user_id == *user_id)
            .cloned();
        match member {
            Some(m) => Ok(m),
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
        let members = self.members.lock().unwrap();
        let filtered: Vec<ServerMember> = members
            .iter()
            .filter(|m| m.server_id == *server_id)
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let offset = (pagination.page - 1) * pagination.limit;
        let paginated: Vec<ServerMember> = filtered
            .into_iter()
            .skip(offset as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated, total))
    }

    async fn update(&self, input: UpdateMemberInput) -> Result<ServerMember, CoreError> {
        let mut members = self.members.lock().unwrap();
        let member = members
            .iter_mut()
            .find(|m| m.server_id == input.server_id && m.user_id == input.user_id);

        match member {
            Some(m) => {
                if let Some(nickname) = input.nickname {
                    m.nickname = Some(nickname);
                }
                m.updated_at = Some(Utc::now());
                Ok(m.clone())
            }
            None => Err(CoreError::MemberNotFound {
                server_id: input.server_id,
                user_id: input.user_id,
            }),
        }
    }

    async fn delete(&self, server_id: &ServerId, user_id: &UserId) -> Result<(), CoreError> {
        let mut members = self.members.lock().unwrap();
        let initial_len = members.len();
        members.retain(|m| !(m.server_id == *server_id && m.user_id == *user_id));

        if members.len() == initial_len {
            Err(CoreError::MemberNotFound {
                server_id: *server_id,
                user_id: *user_id,
            })
        } else {
            Ok(())
        }
    }

    async fn find_by_id(&self, member_id: MemberId) -> Result<ServerMember, CoreError> {
        let members = self.members.lock().unwrap();

        let member = members
            .iter()
            .find(|member| *member.id == *member_id)
            .ok_or(CoreError::MemberNotFoundById { member_id })?;

        Ok(member.to_owned())
    }
}
