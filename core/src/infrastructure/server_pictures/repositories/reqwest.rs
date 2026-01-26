use reqwest::{Client, Url};
use futures_util::future::join_all;

use crate::{
    domain::{
        common::CoreError,
        server::entities::ServerId,
        server_pictures::{Content, ContentVerb, PresignedUrl, ServerPictureUrls, ServerPicturesMap, ServerPicturesRepository},
    },
    infrastructure::server_pictures::repositories::entities::RequestSignUrl,
};

#[derive(Debug, Clone)]
pub struct ReqwestServerPicturesRepository {
    content_url: String,
    client: Client,
}

impl ReqwestServerPicturesRepository {
    pub fn new(content_url: String) -> Self {
        Self {
            content_url,
            client: Client::new(),
        }
    }
}

impl ServerPicturesRepository for ReqwestServerPicturesRepository {
    async fn get_signed_url(
        &self,
        server_id: ServerId,
        content: Content,
        verb: ContentVerb,
    ) -> Result<PresignedUrl, CoreError> {
        let content_url = Url::parse(self.content_url.clone().as_str()).map_err(|_| {
            CoreError::ParseContentUrl {
                part: self.content_url.clone(),
            }
        })?;
        let formatted_prefix = format!("{}/", content);
        let url_with_prefix = content_url.join(formatted_prefix.as_str()).map_err(|_| {
            CoreError::ParseContentUrl {
                part: content.to_string(),
            }
        })?;

        let url = url_with_prefix
            .join(server_id.to_string().as_str())
            .map_err(|_| CoreError::ParseContentUrl {
                part: server_id.to_string(),
            })?;

        let presigned_url = self
            .client
            .post(url)
            .json(&RequestSignUrl::from(verb))
            .send()
            .await
            .map_err(|e| CoreError::FailedToGetSignedUrl { err: e.to_string() })?
            .json::<PresignedUrl>()
            .await
            .map_err(|e| CoreError::FailedToGetSignedUrl { err: e.to_string() })?;

        Ok(presigned_url)
    }

    async fn put_banner(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.get_signed_url(server_id, Content::ServerBanner, ContentVerb::Put)
            .await
    }

    async fn get_banner(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.get_signed_url(server_id, Content::ServerBanner, ContentVerb::Get)
            .await
    }

    async fn get_picture(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.get_signed_url(server_id, Content::ServerPicture, ContentVerb::Get)
            .await
    }

    async fn put_picture(&self, server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        self.get_signed_url(server_id, Content::ServerPicture, ContentVerb::Put)
            .await
    }

    async fn get_all(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        let (banner, picture) = tokio::join!(
            self.get_banner(server_id),
            self.get_picture(server_id)
        );
        Ok(ServerPictureUrls {
            banner: banner?,
            picture: picture?,
        })
    }

    async fn put_all(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        let (banner, picture) = tokio::join!(
            self.put_banner(server_id),
            self.put_picture(server_id)
        );
        Ok(ServerPictureUrls {
            banner: banner?,
            picture: picture?,
        })
    }

    async fn get_all_for_servers(&self, server_ids: Vec<ServerId>) -> ServerPicturesMap {
        let futures = server_ids.into_iter().map(|server_id| async move {
            self.get_all(server_id).await.ok().map(|urls| (server_id, urls))
        });
        join_all(futures).await.into_iter().flatten().collect()
    }
}
