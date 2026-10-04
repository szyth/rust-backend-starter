use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

// SQL files under db/migrations are compiled into the binary, so the container
// image needs no diesel CLI and no migration files at runtime.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Applies pending migrations and returns the names of the ones it ran.
pub fn run() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = crate::pg::get_conn()?;
    let applied = conn.run_pending_migrations(MIGRATIONS)?;
    Ok(applied.iter().map(|m| m.to_string()).collect())
}
