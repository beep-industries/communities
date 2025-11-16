use core::CommunitiesService;


/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: CommunitiesService,
}

impl AppState {
    /// Create a new AppState with the given service
    pub fn new(service: CommunitiesService) -> Self {
        Self { service }
    }
}
