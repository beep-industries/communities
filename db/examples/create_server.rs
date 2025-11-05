use std::sync::Arc;

use db::{database::Database, models::server::CreateServer, repositories};
use uuid::Uuid;

#[tokio::main]
pub async fn main() {
    const DATABASE_URL: &str = "postgres://postgres:password@localhost/communities";
    let owner_id = Uuid::parse_str("64811df3-243e-4833-a235-88c2b2726b06").unwrap();
    // Create a new database connection
    let db = Arc::new(Database::new(DATABASE_URL).await.unwrap());
    // Use the database connection (e.g., create a new server)
    let server_repo = repositories::server::ServerRepository::new(db.clone());
    let new_server = CreateServer {
        name: "My Server".to_string(),
        banner_url: None,
        picture_url: None,
        description: Some("This is my server".to_string()),
        owner_id,
    };

    let created_server = server_repo.create(new_server).await.unwrap();
    println!("Created server: {:?}", created_server);
}
