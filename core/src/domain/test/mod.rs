use crate::{
    Service,
    domain::{
        channel::ports::MockChannelRepository, channel_member::ports::MockChannelMemberRepository,
        friend::ports::MockFriendshipRepository, health::port::MockHealthRepository,
        outbox::ports::MockOutboxRepository, role::ports::MockRoleRepository,
        server::ports::MockServerRepository, server_member::MockMemberRepository,
    },
};
pub mod channel_member;
pub mod channel;
pub mod friend;
pub mod role;
pub mod server;
pub mod server_member;

pub type MockService = Service<
    MockServerRepository,
    MockFriendshipRepository,
    MockHealthRepository,
    MockMemberRepository,
    MockChannelRepository,
    MockRoleRepository,
    MockOutboxRepository,
    MockChannelMemberRepository,
>;

pub fn create_mock_service() -> MockService {
    let channel_repository = MockChannelRepository::new();
    let friendship_repository = MockFriendshipRepository::new();
    let health_repository = MockHealthRepository::new();
    let member_repository = MockMemberRepository::new();
    let server_repository = MockServerRepository::new();
    let role_repository = MockRoleRepository::new();
    let outbox_repository = MockOutboxRepository::new();
    let channel_member_repository = MockChannelMemberRepository::new();
    MockService::new(
        server_repository,
        friendship_repository,
        health_repository,
        member_repository,
        channel_repository,
        role_repository,
        outbox_repository,
        channel_member_repository,
    )
}
