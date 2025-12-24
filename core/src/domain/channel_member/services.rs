use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::{
            entities::{ChannelMember, CreateChannelMemberInput, DeleteChannelMemberInput},
            ports::{ChannelMemberRepository, ChannelMemberService},
        },
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM> ChannelMemberService for Service<S, F, H, M, C, R, O, CM>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
{
    async fn create_channel_member(
        &self,
        input: CreateChannelMemberInput,
    ) -> Result<ChannelMember, CoreError> {
        self.channel_member_repository
            .create(input.user_id, input.channel_id)
            .await
    }

    async fn delete_channel_member(
        &self,
        input: DeleteChannelMemberInput,
    ) -> Result<(), CoreError> {
        self.channel_member_repository
            .delete(input.user_id, input.channel_id)
            .await
    }
}
