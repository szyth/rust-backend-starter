use tokio::task::JoinHandle;

use crate::shared::{constants::SERVER_ERROR, types::ClientError};

/// Awaits a spawned task (typically `spawn_blocking` around a diesel call) and
/// turns a panicked or cancelled task into a server error.
pub async fn flatten_join_handle<T>(
    handle: JoinHandle<Result<T, ClientError>>,
) -> Result<T, ClientError> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(join_err) => {
            tracing::error!("{join_err}");
            Err(ClientError::ServerError(SERVER_ERROR.to_string()))
        }
    }
}
