## DB Setup

```bash
# run from this folder; compose reads ./.env (copy from sample.env)
docker compose up -d postgres

# optional pgAdmin on http://localhost:5050
docker compose --profile tools up -d

# new migration (needs diesel CLI), then regenerate src/schema.rs
diesel migration generate <migration name>
diesel migration run
diesel print-schema > src/schema.rs

# stop
docker compose stop
```

The service applies pending migrations itself at startup (embedded with `diesel_migrations`).
