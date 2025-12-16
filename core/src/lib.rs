pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types for convenience
pub use application::{CommunitiesService, create_repositories};
pub use domain::common::services::Service;
pub use infrastructure::channel::repositories::PostgresChannelRepository;
pub use infrastructure::friend::repositories::postgres::PostgresFriendshipRepository;
pub use infrastructure::health::repositories::postgres::PostgresHealthRepository;
pub use infrastructure::server::repositories::postgres::PostgresServerRepository;

// Re-export outbox pattern primitives
pub use infrastructure::outbox::write_outbox_event;
