use crate::{
    Service,
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::{entities::ServerId, ports::ServerRepository},
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        server_pictures::{Content, ContentVerb, ServerPicturesRepository, ServerPicturesService},
        user::port::UserRepository,
    },
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> ServerPicturesService
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
    async fn put_server_banner(&self, server_id: ServerId) {
        self.server_pictures_repository
            .get_signed_url(server_id, Content::ServerBanner, ContentVerb::Put)
            .await
    }

    async fn get_server_banner(&self, server_id: ServerId) {
        self.server_pictures_repository
            .get_signed_url(server_id, Content::ServerBanner, ContentVerb::Get)
            .await
    }
    async fn put_server_picture(&self, server_id: ServerId) {
        self.server_pictures_repository
            .get_signed_url(server_id, Content::ServerPicture, ContentVerb::Put)
            .await
    }

    async fn get_server_picture(&self, server_id: ServerId) {
        self.server_pictures_repository
            .get_signed_url(server_id, Content::ServerPicture, ContentVerb::Get)
            .await
    }
}
