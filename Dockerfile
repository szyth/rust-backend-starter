# syntax=docker/dockerfile:1

# ---- build stage: full Rust toolchain, never shipped ----
FROM rust:1-slim-trixie AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
# cache mounts keep the crate registry and target dir between builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked -p service \
    && cp target/release/service /usr/local/bin/service

# ---- runtime stage: only the binary and libpq ----
FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home app
COPY --from=builder /usr/local/bin/service /usr/local/bin/service
# run as a non-root user
USER 10001
ENV PORT=8000 \
    LOG_FORMAT=json
EXPOSE 8000
CMD ["service"]
