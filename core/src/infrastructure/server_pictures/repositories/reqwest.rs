use crate::domain::server_pictures::ServerPicturesRepository;

#[derive(Debug, Clone)]
pub struct ReqwestServerPicturesRepository {}

impl ServerPicturesRepository for ReqwestServerPicturesRepository {
    async fn get_signed_url(
        server_id: crate::domain::server::entities::ServerId,
        content: crate::domain::server_pictures::Content,
        verb: crate::domain::server_pictures::ContentVerb,
    ) {
    }
}
