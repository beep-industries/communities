use crate::{
    Service,
    domain::{
        authorization::ports::MockAuthorizationRepository,
        channel::ports::MockChannelRepository,
        channel_member::ports::MockChannelMemberRepository,
        friend::ports::MockFriendshipRepository,
        health::port::MockHealthRepository,
        member_role::ports::MockMemberRoleRepository,
        outbox::ports::MockOutboxRepository,
        role::ports::MockRoleRepository,
        server::ports::MockServerRepository,
        server_invitation::ports::MockServerInvitationRepository,
        server_member::MockMemberRepository,
        server_pictures::{self, MockServerPicturesRepository},
        user::port::MockUserRepository,
    },
};
pub mod channel;
pub mod channel_member;
pub mod friend;
pub mod member_role;
pub mod role;
pub mod server;
pub mod server_member;

pub type MockService = Service<
    MockServerRepository,
    MockFriendshipRepository,
    MockUserRepository,
    MockHealthRepository,
    MockMemberRepository,
    MockChannelRepository,
    MockRoleRepository,
    MockOutboxRepository,
    MockChannelMemberRepository,
    MockMemberRoleRepository,
    MockServerInvitationRepository,
    MockAuthorizationRepository,
    MockServerPicturesRepository,
>;

pub fn create_mock_service() -> MockService {
    let channel_repository = MockChannelRepository::new();
    let friendship_repository = MockFriendshipRepository::new();
    let user_repository = MockUserRepository::new();
    let health_repository = MockHealthRepository::new();
    let member_repository = MockMemberRepository::new();
    let server_repository = MockServerRepository::new();
    let role_repository = MockRoleRepository::new();
    let outbox_repository = MockOutboxRepository::new();
    let channel_member_repository = MockChannelMemberRepository::new();
    let member_role_repository = MockMemberRoleRepository::new();
    let server_invitation_repository = MockServerInvitationRepository::new();
    let authorization_repository = MockAuthorizationRepository::new();
    let server_pictures_repository = MockServerPicturesRepository::new();
    MockService::new(
        server_repository,
        friendship_repository,
        user_repository,
        health_repository,
        member_repository,
        channel_repository,
        role_repository,
        outbox_repository,
        channel_member_repository,
        member_role_repository,
        server_invitation_repository,
        authorization_repository,
        server_pictures_repository,
    )
}
