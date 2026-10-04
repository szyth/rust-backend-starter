use std::sync::LazyLock;
use std::time::Duration;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection, RunQueryDsl,
};

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error("DieselError: {:?}", _0)]
    Diesel(#[from] diesel::result::Error),
    #[error("PooledConnectionError: cannot get the connection from r2d2 pool: {0}")]
    PooledConnection(#[from] diesel::r2d2::PoolError),
}

impl DBError {
    /// True when an insert/update hit a UNIQUE constraint (e.g. duplicate email).
    pub fn is_unique_violation(&self) -> bool {
        matches!(
            self,
            DBError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            ))
        )
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

// Pooled Database connection. Performance improvement
// https://en.wikipedia.org/wiki/Connection_pool
fn build_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var not found");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .test_on_check_out(true)
        // fail fast: a readiness probe must answer before Kubernetes times it out
        .connection_timeout(Duration::from_secs(3))
        // don't open connections at startup, so the process starts even if the DB is
        // briefly unavailable; /ready reports it instead of the pod crash-looping
        .build_unchecked(manager)
}

static DB_POOL: LazyLock<PgPool> = LazyLock::new(build_pool);

pub fn get_conn() -> Result<PooledConnection<ConnectionManager<PgConnection>>, DBError> {
    DB_POOL.get().map_err(|e| {
        tracing::error!("DB_ERROR: Not able to get the connection from pool: {e}");
        DBError::from(e)
    })
}

/// Cheapest possible round trip, used by the readiness probe.
pub fn ping() -> Result<(), DBError> {
    let mut conn = get_conn()?;
    diesel::sql_query("SELECT 1").execute(&mut conn)?;
    Ok(())
}
