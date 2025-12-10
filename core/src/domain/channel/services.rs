use crate::{
    Service,
    domain::{
        channel::{
            entities::{
                Channel, ChannelError, ChannelType, CreateChannelInput, CreatePrivateChannelInput,
                CreateServerChannelInput,
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
        create_channel_input: CreatePrivateChannelInput,
    ) -> Result<Channel, CoreError> {
        let channel_name = create_channel_input.name.trim().to_string();
        
        if channel_name.len() > 30 {
            return Err(ChannelError::IncorrectChannelPayload {
                msg: "The name of the channel is too long".into(),
            }
            .into());
        }

        let repo_channel_input = CreateChannelInput {
            name: channel_name,
            server_id: None,
            parent_id: None,
            channel_type: ChannelType::Private,
        };
        
        self.channel_repository.create(repo_channel_input).await
    }

    async fn create_server_channel(
        &self,
        create_channel_input: CreateServerChannelInput,
    ) -> Result<Channel, CoreError> {
        todo!()
    }

    async fn list_channels_in_server(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<Channel>, CoreError> {
        todo!()
    }

    async fn update_channel(
        &self,
        update_channel_input: super::entities::UpdateChannelInput,
    ) -> Result<Channel, CoreError> {
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
