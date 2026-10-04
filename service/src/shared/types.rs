#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Bad Request: {0}")]
    BadRequestError(String),
    #[error("Not Found: {0}")]
    NotFoundError(String),
    #[error("Conflict: {0}")]
    ConflictError(String),
    #[error("Unauthorised: {0}")]
    UnauthorisedError(String),
    #[error("Service Unavailable: {0}")]
    UnavailableError(String),
    #[error("Server Error: {0}")]
    ServerError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    #[error("CustomError: {0}")]
    Custom(String),
    #[error("JsonParseError: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("HyperError: {0}")]
    HyperError(#[from] hyper::Error),
}
