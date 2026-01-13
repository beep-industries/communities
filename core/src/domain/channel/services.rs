use crate::{
    Service,
    domain::{
        channel::{
            entities::{
                Channel, ChannelError, ChannelId, ChannelType, CreateChannelRepoInput,
                CreatePrivateChannelInput, CreateServerChannelInput, UpdateChannelInput,
            },
            ports::{ChannelRepository, ChannelService},
        },
        channel_member::ports::ChannelMemberRepository,
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::{entities::ServerId, ports::ServerRepository},
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM, MR> ChannelService for Service<S, F, H, M, C, R, O, CM, MR>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
{
    async fn create_private_channel(
        &self,
        mut create_channel_input: CreatePrivateChannelInput,
    ) -> Result<Channel, CoreError> {
        let channel_name = create_channel_input.name.value()?;

        let repo_channel_input = CreateChannelRepoInput {
            name: channel_name,
            server_id: None,
            parent_id: None,
            channel_type: ChannelType::Private,
        };

        self.channel_repository.create(repo_channel_input).await
    }

    async fn create_server_channel(
        &self,
        mut create_channel_input: CreateServerChannelInput,
    ) -> Result<Channel, CoreError> {
        let channel_name = create_channel_input.name.value()?;
        // It should only be server type
        let channel_type = match create_channel_input.channel_type {
            ChannelType::ServerFolder | ChannelType::ServerText | ChannelType::ServerVoice => {
                create_channel_input.channel_type
            }
            _ => return Err(ChannelError::WrongChannelType.into()),
        };

        // TODO: Verify and use the parent id with the get channel function
        let repo_channel_input = CreateChannelRepoInput {
            name: channel_name,
            server_id: Some(create_channel_input.server_id),
            parent_id: create_channel_input.parent_id,
            channel_type,
        };

        self.channel_repository.create(repo_channel_input).await
    }

    async fn list_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<Channel>, CoreError> {
        self.channel_repository.list_in_server(server_id).await
    }

    async fn update_channel(
        &self,
        mut update_channel_input: UpdateChannelInput,
    ) -> Result<Channel, CoreError> {
        let repo_input = update_channel_input.into_repo_input()?;
        self.channel_repository.update(repo_input).await
    }

    async fn delete_channel(&self, channel_id: ChannelId) -> Result<(), CoreError> {
        self.channel_repository.delete(channel_id).await
    }

    async fn get_channel_by_id(&self, channel_id: ChannelId) -> Result<Channel, CoreError> {
        self.channel_repository.find_by_id(channel_id).await
    }
}
