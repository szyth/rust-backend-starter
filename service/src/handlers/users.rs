use serde::Deserialize;

use crate::shared::{
    constants::{BAD_REQUEST_ERROR, EMAIL_ALREADY_EXISTS, SERVER_ERROR},
    types::ClientError,
};
use crate::utils::functions::flatten_join_handle;

#[derive(Deserialize)]
pub(crate) struct CreateUserRequest {
    name: String,
    email: String,
}

pub(crate) async fn create(req: CreateUserRequest) -> Result<db::models::users::User, ClientError> {
    if req.name.trim().is_empty() || !req.email.contains('@') {
        return Err(ClientError::BadRequestError(BAD_REQUEST_ERROR.to_string()));
    }

    // diesel is synchronous: run it on the blocking pool, not a Tokio worker thread
    flatten_join_handle(tokio::task::spawn_blocking(move || {
        db::models::users::insert(req.name.trim(), req.email.trim()).map_err(|e| {
            if e.is_unique_violation() {
                ClientError::ConflictError(EMAIL_ALREADY_EXISTS.to_string())
            } else {
                tracing::error!("create user failed: {e}");
                ClientError::ServerError(SERVER_ERROR.to_string())
            }
        })
    }))
    .await
}
