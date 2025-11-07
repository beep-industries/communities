use crate::domain::server::ports::ServerRepository;

pub struct Service<S>
where
    S: ServerRepository,
{
    pub(crate) server_repository: S,
}

impl<S> Service<S>
where
    S: ServerRepository,
{
    pub fn new(server_repository: S) -> Self {
        Self { server_repository }
    }
}
