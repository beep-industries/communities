use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::{
    channel_member::entities::{ChannelMember, CreateChannelMemberInput, DeleteChannelMemberInput},
    common::CoreError,
};

// This only concern Private channel
pub trait ChannelMemberService: Send + Sync {
    fn create_channel_member(
        &self,
        input: CreateChannelMemberInput,
    ) -> impl Future<Output = Result<ChannelMember, CoreError>> + Send;
    fn delete_channel_member(
        &self,
        input: DeleteChannelMemberInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

// This only concern Private channel
pub trait ChannelMemberRepository: Send + Sync {
    fn create(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> impl Future<Output = Result<ChannelMember, CoreError>> + Send;
    fn delete(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[derive(Clone, Debug)]
pub struct MockChannelMemberRepository {
    pub members: Arc<Mutex<Vec<ChannelMember>>>,
}

impl MockChannelMemberRepository {
    pub fn new() -> Self {
        Self {
            members: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ChannelMemberRepository for MockChannelMemberRepository {
    fn create(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> impl Future<Output = Result<ChannelMember, CoreError>> + Send {
        let members = self.members.clone();
        async move {
            let mut members = members.lock().unwrap();
            if members
                .iter()
                .any(|m| m.user_id == user_id && m.channel_id == channel_id)
            {
                return Err(CoreError::Error {
                    msg: "Conflict: Channel member already exists".to_string(),
                });
            }
            let new_member = ChannelMember {
                user_id,
                channel_id,
                created_at: chrono::Utc::now(),
                updated_at: Some(chrono::Utc::now()),
            };
            members.push(new_member.clone());
            Ok(new_member)
        }
    }

    fn delete(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send {
        let members = self.members.clone();
        async move {
            let mut members = members.lock().unwrap();
            let initial_len = members.len();
            members.retain(|m| !(m.user_id == user_id && m.channel_id == channel_id));
            if members.len() == initial_len {
                return Err(CoreError::Error {
                    msg: "Not Found: Channel member does not exist".to_string(),
                });
            }
            Ok(())
        }
    }
}
