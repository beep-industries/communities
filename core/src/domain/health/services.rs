use crate::domain::{
    channel::ports::ChannelRepository,
    channel_member::ports::ChannelMemberRepository,
    common::{CoreError, services::Service},
    friend::ports::FriendshipRepository,
    health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    },
    member_role::ports::MemberRoleRepository,
    outbox::ports::OutboxRepository,
    role::ports::RoleRepository,
    server::ports::ServerRepository,
    server_invitation::ports::ServerInvitationRepository,
    server_member::ports::MemberRepository,
    user::port::UserRepository,
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI> HealthService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI>
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
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
