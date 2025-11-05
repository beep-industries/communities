use sqlx::{Pool, Postgres};

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = Pool::<Postgres>::connect(database_url).await?;
        Ok(Self { pool })
    }
}
