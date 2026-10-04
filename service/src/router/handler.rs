use http_body_util::Full;
use hyper::Method;
use hyper::{body::Bytes, Response};
use hyper::{body::Incoming as IncomingBody, Request};

use crate::router::utils::{error, from_body, handle_error, success};

pub async fn handler(
    req: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, service::shared::types::RouteError> {
    let (p, body) = req.into_parts();
    let start = std::time::Instant::now();
    let response = match (&p.method, p.uri.path()) {
        // probes: Kubernetes liveness and readiness
        (&Method::GET, "/health") => success(service::handlers::health::health().await),
        (&Method::GET, "/ready") => match service::handlers::ready::handle().await {
            Ok(ready) => success(ready),
            Err(e) => handle_error(e),
        },

        // users
        (&Method::POST, "/v1/users") => {
            match from_body(Request::from_parts(p.clone(), body)).await {
                Ok(req) => match service::handlers::users::create(req).await {
                    Ok(user) => success(user),
                    Err(e) => handle_error(e),
                },
                Err(e) => handle_error(e),
            }
        }

        // Handle CORS preflight request
        (&hyper::Method::OPTIONS, _) => {
            let mut response = hyper::Response::default();
            crate::middleware::cors::add_headers(&mut response);
            *response.status_mut() = hyper::StatusCode::NO_CONTENT;
            Ok(response)
        }
        _ => error(
            format!("NOT_FOUND method: {}, path: {}", p.method, p.uri.path()),
            hyper::StatusCode::NOT_FOUND,
        ),
    };

    // one structured access-log line per request
    if let Ok(res) = &response {
        tracing::info!(
            method = %p.method,
            path = p.uri.path(),
            status = res.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            "request"
        );
    }
    response
}
