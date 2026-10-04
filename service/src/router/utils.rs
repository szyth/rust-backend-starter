use http_body_util::{BodyExt, Full, Limited};
use hyper::{body::Bytes, Response};
use hyper::{body::Incoming as IncomingBody, Request};

use crate::shared::constants::{BAD_REQUEST_ERROR, MAX_BODY_BYTES};
use crate::shared::types::ClientError;

// Goal: Convert hyper::body::Incoming -> Generic T, where T represents Request{} for each API handler
// steps: first convert Request to Bytes (capped at MAX_BODY_BYTES)
// then Bytes to generic struct T using serde_json::from_slice()
pub async fn from_body<T: serde::de::DeserializeOwned>(
    req: Request<IncomingBody>,
) -> Result<T, ClientError> {
    let req_bytes = Limited::new(req.into_body(), MAX_BODY_BYTES)
        .collect()
        .await
        .map_err(|e| {
            tracing::warn!("request body rejected: {e}");
            ClientError::BadRequestError(BAD_REQUEST_ERROR.to_string())
        })?
        .to_bytes();

    // Deserialize JSON into a Struct T; malformed input is the client's fault (400)
    serde_json::from_slice(req_bytes.as_ref()).map_err(|e| {
        tracing::warn!("invalid JSON body: {e}");
        ClientError::BadRequestError(BAD_REQUEST_ERROR.to_string())
    })
}

pub fn success<T: serde::Serialize>(
    data: T,
) -> Result<Response<Full<Bytes>>, crate::shared::types::RouteError> {
    #[derive(serde::Serialize)]
    struct ApiSuccess<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let resp = serde_json::to_vec(&ApiSuccess {
        data,
        success: true,
    })?;

    let mut response = hyper::Response::new(Full::from(resp));
    *response.status_mut() = hyper::StatusCode::OK;
    crate::middleware::cors::add_headers(&mut response);
    Ok(response)
}

pub fn error(
    message: String,
    status: hyper::StatusCode,
) -> Result<Response<Full<Bytes>>, crate::shared::types::RouteError> {
    Ok(error_without_result(message, status))
}

pub fn handle_error(
    e: ClientError,
) -> Result<Response<Full<Bytes>>, crate::shared::types::RouteError> {
    let (msg, status_code) = match e {
        ClientError::BadRequestError(error_msg) => (error_msg, hyper::StatusCode::BAD_REQUEST),
        ClientError::NotFoundError(error_msg) => (error_msg, hyper::StatusCode::NOT_FOUND),
        ClientError::ConflictError(error_msg) => (error_msg, hyper::StatusCode::CONFLICT),
        ClientError::UnauthorisedError(error_msg) => (error_msg, hyper::StatusCode::UNAUTHORIZED),
        ClientError::UnavailableError(error_msg) => {
            (error_msg, hyper::StatusCode::SERVICE_UNAVAILABLE)
        }
        ClientError::ServerError(error_msg) => {
            (error_msg, hyper::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    error(msg, status_code)
}

pub fn error_without_result(message: String, status: hyper::StatusCode) -> Response<Full<Bytes>> {
    #[derive(serde::Serialize)]
    struct ApiError {
        message: String,
        success: bool,
    }
    if status.is_server_error() {
        tracing::error!(message);
    }
    let resp = serde_json::to_vec(&ApiError {
        success: false,
        message,
    })
    .unwrap();

    let mut response = Response::new(Full::from(resp));
    *response.status_mut() = status;
    crate::middleware::cors::add_headers(&mut response);
    response
}
