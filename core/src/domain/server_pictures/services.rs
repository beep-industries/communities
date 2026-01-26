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
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        server_pictures::{ServerPicturesRepository, ServerPicturesService},
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
    fn put_server_banner(
        server_id: crate::domain::server::entities::ServerId,
    ) -> impl Future<Output = ()> {
        todo!()
    }

    fn get_server_banner(
        server_id: crate::domain::server::entities::ServerId,
    ) -> impl Future<Output = ()> {
        todo!()
    }

    fn put_server_picture(
        server_id: crate::domain::server::entities::ServerId,
    ) -> impl Future<Output = ()> {
        todo!()
    }

    fn get_server_picture(
        server_id: crate::domain::server::entities::ServerId,
    ) -> impl Future<Output = ()> {
        todo!()
    }
}
