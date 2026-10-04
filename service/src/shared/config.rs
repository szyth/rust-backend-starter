// All configuration comes from environment variables. Locally they can be put in
// an optional `<ENV>.env` file; in Kubernetes they come from the pod spec.

pub fn env() -> Option<String> {
    std::env::var("ENV").ok().map(|x| x.to_lowercase())
}

/// Loads `<ENV>.env` if it exists. Variables already set in the environment win,
/// so the pod spec always overrides the file.
pub fn load_env_file() {
    if let Some(env) = env() {
        let _ = dotenvy::from_filename(format!("{env}.env"));
    }
}

/// `LOG_FORMAT=json` for CloudWatch; anything else gives the pretty local output.
pub fn log_json() -> bool {
    std::env::var("LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json"))
}

pub fn log_level() -> tracing_subscriber::filter::LevelFilter {
    std::env::var("LOG_LEVEL")
        .ok()
        .and_then(|l| l.parse().ok())
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO)
}

// if no port is provided, take 8000 as default
pub fn port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000)
}

/// Set `RUN_MIGRATIONS=false` when migrations run as a separate job instead.
pub fn run_migrations() -> bool {
    !std::env::var("RUN_MIGRATIONS").is_ok_and(|v| v.eq_ignore_ascii_case("false"))
}
