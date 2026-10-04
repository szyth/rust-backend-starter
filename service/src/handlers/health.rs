// Liveness: "is the process alive?" Deliberately does NOT touch the database,
// otherwise a database outage would make Kubernetes restart every pod.
#[derive(serde::Serialize)]
pub struct Health {
    status: &'static str,
    version: &'static str,
    env: String,
}

pub async fn health() -> Health {
    let env = service::shared::config::env().unwrap_or_else(|| "unset".to_owned());
    Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        env,
    }
}
