use crate::domain::{
    server::entities::ServerId,
    server_pictures::{Content, ContentVerb},
};

pub trait ServerPicturesRepository: Send + Sync {
    fn get_signed_url(
        server_id: ServerId,
        content: Content,
        verb: ContentVerb,
    ) -> impl Future<Output = ()>;
}

pub trait ServerPicturesService: Send + Sync {
    fn put_server_banner(server_id: ServerId) -> impl Future<Output = ()>;
    fn get_server_banner(server_id: ServerId) -> impl Future<Output = ()>;
    fn put_server_picture(server_id: ServerId) -> impl Future<Output = ()>;
    fn get_server_picture(server_id: ServerId) -> impl Future<Output = ()>;
}

pub struct MockServerPicturesRepository;

impl MockServerPicturesRepository {
    pub fn new() -> Self {
        MockServerPicturesRepository
    }
}

impl ServerPicturesRepository for MockServerPicturesRepository {
    async fn get_signed_url(server_id: ServerId, content: Content, verb: ContentVerb) {}
}
