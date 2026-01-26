use crate::domain::{server::entities::ServerId, server_pictures::Content};

pub trait ServerPicturesRepository: Send + Sync {
    fn get_signed_url(server_id: ServerId, content: Content) -> impl Future<Output = ()>;
}

pub trait ServerPicturesService: Send + Sync {
    fn put_server_banner(server_id: ServerId) -> impl Future<Output = ()>;
    fn get_server_banner(server_id: ServerId) -> impl Future<Output = ()>;
    fn put_server_picture(server_id: ServerId) -> impl Future<Output = ()>;
    fn get_server_picture(server_id: ServerId) -> impl Future<Output = ()>;
}
