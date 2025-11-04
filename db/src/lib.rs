use std::time::Duration;

use sqlx::{
    migrate::{MigrateError, Migrator},
    postgres::PgPoolOptions,
    PgPool,
};
use thiserror::Error;

/// Exposes the compiled SQLx migrator for reuse in other crates.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Unifies the possible failure modes encountered while preparing the database.
#[derive(Debug, Error)]
pub enum DbError {
    /// Wrapper for any connection or pool related error encountered by SQLx.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// Wrapper for migration failures (missing scripts, applied-with-drift, etc.).
    #[error(transparent)]
    Migration(#[from] MigrateError),
}

/// Establish a PostgreSQL connection pool using the provided connection string.
///
/// The pool is configured with sane defaults for server-side workloads; callers can
/// further tweak connection settings by building their own [`PgPoolOptions`].
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
}

/// Establish a connection pool and run all pending migrations in a single call.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, DbError> {
    let pool = connect(database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

/// Execute all pending migrations against the provided connection pool.
pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_embedded() {
        // The migrator should be aware of at least one migration file.
        assert!(!MIGRATOR.migrations.is_empty());
    }
}
