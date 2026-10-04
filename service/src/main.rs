use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use service::router::utils::error_without_result;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Kubernetes waits terminationGracePeriodSeconds (30s by default) before SIGKILL;
// finish draining well inside that.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(20);
const MIGRATION_ATTEMPTS: u32 = 10;
const MIGRATION_RETRY_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
struct HttpService {}

impl Service<Request<IncomingBody>> for HttpService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        Box::pin(async {
            match service::router::handler::handler(req).await {
                Ok(res) => Ok(res),
                Err(err) => {
                    tracing::error!(message = format!("handler-error: {}", err));

                    Ok(error_without_result(
                        "INTERNAL_SERVER_ERROR".to_owned(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        })
    }
}

fn main() {
    // optional local <ENV>.env file; must run before logging reads LOG_FORMAT/LOG_LEVEL
    service::shared::config::load_env_file();

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(traced_main());

    if let Err(e) = result {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
}

async fn traced_main() -> Result<(), BoxError> {
    use tracing_subscriber::layer::SubscriberExt;
    let level = service::shared::config::log_level();

    if service::shared::config::log_json() {
        // one JSON object per line: what CloudWatch Logs Insights queries best
        tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_max_level(level)
            .init();
        http_main().await
    } else {
        tracing_forest::worker_task()
            .set_global(true)
            .build_with(|layer: tracing_forest::ForestLayer<_, _>| {
                tracing_subscriber::Registry::default()
                    .with(layer)
                    .with(level)
            })
            .on(http_main())
            .await
    }
}

async fn http_main() -> Result<(), BoxError> {
    if std::env::var("DATABASE_URL").is_err() {
        return Err("DATABASE_URL must be set".into());
    }

    if service::shared::config::run_migrations() {
        run_migrations_with_retry().await?;
    }

    let socket_address: std::net::SocketAddr =
        ([0, 0, 0, 0], service::shared::config::port()).into();
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    tracing::info!(
        "#### Started Server at: {}:{} ####",
        socket_address.ip(),
        socket_address.port()
    );

    let graceful = GracefulShutdown::new();
    let mut shutdown = std::pin::pin!(shutdown_signal());

    loop {
        tokio::select! {
            // Handle incoming HTTP connections
            result = listener.accept() => {
                let (stream, _) = match result {
                    Ok(conn) => conn,
                    Err(e) => {
                        tracing::warn!("accept failed: {e}");
                        continue;
                    }
                };
                let io = TokioIo::new(stream);

                // graceful.watch lets us wait for in-flight requests on shutdown
                let conn = graceful.watch(http1::Builder::new().serve_connection(io, HttpService {}));
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::warn!("Failed to serve connection: {err:?}");
                    }
                });
            }

            // SIGTERM (Kubernetes rollout/scale-down) or Ctrl+C: stop accepting, then drain
            _ = &mut shutdown => {
                tracing::info!("shutdown signal received, draining connections");
                break;
            }
        }
    }

    drop(listener);
    tokio::select! {
        _ = graceful.shutdown() => tracing::info!("all connections closed, exiting"),
        _ = tokio::time::sleep(DRAIN_TIMEOUT) => tracing::warn!("drain timed out, exiting"),
    }
    Ok(())
}

/// Runs embedded migrations, retrying while the database is still starting
/// (common with docker compose, and when RDS is restarting).
async fn run_migrations_with_retry() -> Result<(), BoxError> {
    for attempt in 1..=MIGRATION_ATTEMPTS {
        match tokio::task::spawn_blocking(db::migrations::run).await? {
            Ok(applied) => {
                tracing::info!(?applied, "database migrations up to date");
                return Ok(());
            }
            Err(e) if attempt < MIGRATION_ATTEMPTS => {
                tracing::warn!(attempt, "database not ready, retrying migrations: {e}");
                tokio::time::sleep(MIGRATION_RETRY_DELAY).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!("loop returns on success or final failure")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
