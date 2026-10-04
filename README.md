# rust-backend-starter

Boilerplate for Rust backend services: hyper 1.x + Tokio + Diesel (Postgres), container-ready and Kubernetes-friendly (probes, graceful shutdown, JSON logs, embedded migrations).

```
service/   HTTP server: routing, handlers, config, graceful shutdown
db/        Diesel schema, models, connection pool, embedded migrations
```

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Liveness: process is up. Never touches the DB |
| GET | `/ready` | Readiness: DB reachable (`SELECT 1`). 503 when not |

## Configuration (environment variables)

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | required | `postgres://user:pass@host:5432/db` |
| `PORT` | `8000` | |
| `LOG_FORMAT` | pretty | `json` for CloudWatch (set in the image) |
| `LOG_LEVEL` | `info` | |
| `RUN_MIGRATIONS` | `true` | `false` when a separate job runs migrations |
| `ENV` | unset | Loads an optional `<ENV>.env` file (local dev only) |

## Run locally

```bash
# macOS: Homebrew libpq is keg-only, tell pq-sys where it is
export PQ_LIB_DIR="$(brew --prefix libpq)/lib"

cp db/sample.env db/.env               # fill in local passwords
cd db && docker compose up -d postgres && cd ..
ENV=dev cargo run -p service            # reads dev.env: PORT, DATABASE_URL
curl localhost:8000/health
curl localhost:8000/ready
```

Migrations under `db/migrations` are embedded in the binary and applied at startup.

## Container

```bash
docker buildx build --load -t rust-api:local .
cd db && docker compose --profile app up --build   # service + postgres
```

Multi-stage build, ~120 MB, runs as uid 10001, JSON logs, handles SIGTERM by draining connections.
For x86 EKS nodes from an Apple Silicon Mac: `docker buildx build --platform linux/amd64 ...`.
