use reqwest::{Client, Url};

use crate::{
    domain::{
        common::CoreError,
        server::entities::ServerId,
        server_pictures::{Content, ContentVerb, PresignedUrl, ServerPicturesRepository},
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
}
