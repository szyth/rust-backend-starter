use crate::shared::{constants::DB_NOT_READY, types::ClientError};
use crate::utils::functions::flatten_join_handle;

// Readiness: "can this pod serve traffic?" Checks the database. When it fails,
// Kubernetes stops sending traffic to the pod but does not restart it.
#[derive(serde::Serialize)]
pub struct Ready {
    status: &'static str,
}

pub async fn handle() -> Result<Ready, ClientError> {
    // diesel is synchronous: run it on the blocking pool, not a Tokio worker thread
    flatten_join_handle(tokio::task::spawn_blocking(|| {
        db::pg::ping().map_err(|e| {
            tracing::warn!("readiness check failed: {e}");
            ClientError::UnavailableError(DB_NOT_READY.to_string())
        })
    }))
    .await?;
    Ok(Ready { status: "ready" })
}
