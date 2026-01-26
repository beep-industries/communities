use crate::{
    Service,
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::{entities::ServerId, ports::ServerRepository},
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        server_pictures::{PresignedUrl, ServerPictureUrls, ServerPicturesMap, ServerPicturesRepository, ServerPicturesService},
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
    async fn put_server_banner(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.server_pictures_repository.put_banner(server_id).await
    }

    async fn get_server_banner(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.server_pictures_repository.get_banner(server_id).await
    }
    async fn put_server_picture(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.server_pictures_repository.put_picture(server_id).await
    }

    async fn get_server_picture(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.server_pictures_repository.get_picture(server_id).await
    }

    async fn get_all_server_pictures(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        self.server_pictures_repository.get_all(server_id).await
    }

    async fn put_all_server_pictures(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        self.server_pictures_repository.put_all(server_id).await
    }

    async fn get_all_server_pictures_for_servers(&self, server_ids: Vec<ServerId>) -> ServerPicturesMap {
        self.server_pictures_repository.get_all_for_servers(server_ids).await
    }
}
