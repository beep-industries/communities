use reqwest::Url;
use futures_util::future::join_all;

use crate::domain::{
    common::CoreError,
    server::entities::ServerId,
    server_pictures::{Content, ContentVerb, PresignedUrl, ServerPictureUrls, ServerPicturesMap},
};

pub trait ServerPicturesRepository: Send + Sync {
    fn get_signed_url(
        &self,
        server_id: ServerId,
        content: Content,
        verb: ContentVerb,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn put_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn put_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_all(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<ServerPictureUrls, CoreError>> + Send;

    fn put_all(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<ServerPictureUrls, CoreError>> + Send;

    fn get_all_for_servers(
        &self,
        server_ids: Vec<ServerId>,
    ) -> impl Future<Output = ServerPicturesMap> + Send;
}

pub trait ServerPicturesService: Send + Sync {
    fn put_server_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_server_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_server_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;
    fn put_server_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn get_all_server_pictures(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<ServerPictureUrls, CoreError>> + Send;

    fn put_all_server_pictures(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<ServerPictureUrls, CoreError>> + Send;

    fn get_all_server_pictures_for_servers(
        &self,
        server_ids: Vec<ServerId>,
    ) -> impl Future<Output = ServerPicturesMap> + Send;
}

pub struct MockServerPicturesRepository;

impl MockServerPicturesRepository {
    pub fn new() -> Self {
        MockServerPicturesRepository
    }
}

impl ServerPicturesRepository for MockServerPicturesRepository {
    async fn get_signed_url(
        &self,
        _server_id: ServerId,
        _content: Content,
        _verb: ContentVerb,
    ) -> Result<PresignedUrl, CoreError> {
        Ok(PresignedUrl::new(
            Url::parse("https://example.com").unwrap(),
        ))
    }

    async fn put_banner(&self, _server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        Ok(PresignedUrl::new(
            Url::parse("https://example.com").unwrap(),
        ))
    }

    async fn get_banner(&self, _server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        Ok(PresignedUrl::new(
            Url::parse("https://example.com").unwrap(),
        ))
    }

    async fn get_picture(&self, _server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        Ok(PresignedUrl::new(
            Url::parse("https://example.com").unwrap(),
        ))
    }

    async fn put_picture(&self, _server_id: ServerId) -> Result<PresignedUrl, CoreError> {
        Ok(PresignedUrl::new(
            Url::parse("https://example.com").unwrap(),
        ))
    }

    async fn get_all(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        let (banner, picture) =
            tokio::join!(self.get_banner(server_id), self.get_picture(server_id));
        Ok(ServerPictureUrls {
            banner: banner?,
            picture: picture?,
        })
    }

    async fn put_all(&self, server_id: ServerId) -> Result<ServerPictureUrls, CoreError> {
        let (banner, picture) =
            tokio::join!(self.put_banner(server_id), self.put_picture(server_id));
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
