use crate::{
    Service,
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::{
            entities::{ChannelMember, CreateChannelMemberInput, DeleteChannelMemberInput},
            ports::{ChannelMemberRepository, ChannelMemberService},
        },
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        server_pictures::ServerPicturesRepository,
        user::port::UserRepository,
    },
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> ChannelMemberService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
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
