use reqwest::Url;

use crate::domain::{
    common::CoreError,
    server::entities::ServerId,
    server_pictures::{Content, ContentVerb, PresignedUrl},
};

pub trait ServerPicturesRepository: Send + Sync {
    fn get_signed_url(
        &self,
        server_id: ServerId,
        content: Content,
        verb: ContentVerb,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn put_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn get_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn get_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn put_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;
}

pub trait ServerPicturesService: Send + Sync {
    fn put_server_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn get_server_banner(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;

    fn get_server_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;
    fn put_server_picture(
        &self,
        server_id: ServerId,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>>;
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
}
