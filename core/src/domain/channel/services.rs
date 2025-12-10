use crate::{
    Service,
    domain::{
        channel::{
            entities::{
                Channel, ChannelError, ChannelType, CreateChannelRepoInput,
                CreatePrivateChannelInput, CreateServerChannelInput, UpdateChannelInput,
            },
            ports::{ChannelRepository, ChannelService},
        },
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        server::{entities::ServerId, ports::ServerRepository},
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C> ChannelService for Service<S, F, H, M, C>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
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
            parent_id: None,
            channel_type,
        };

        self.channel_repository.create(repo_channel_input).await
    }

    async fn list_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<Channel>, CoreError> {
        todo!()
    }

    async fn update_channel(
        &self,
        update_channel_input: UpdateChannelInput,
    ) -> Result<Channel, CoreError> {
        // if update_channel_input.is_empty() {
        //     return Err(ChannelError::EmptyUpdatePayload.into());
        // }

        // if channel_name.trim() > MAX_CHANNEL_NAME_SIZE {
        //     return Err(ChannelError::ChannelNameTooLong.into());
        // }
        todo!()
    }

    async fn delete_channel(
        &self,
        channel_id: super::entities::ChannelId,
    ) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_channel_by_id(
        &self,
        channel_id: super::entities::ChannelId,
    ) -> Result<Channel, CoreError> {
        todo!()
    }
}
