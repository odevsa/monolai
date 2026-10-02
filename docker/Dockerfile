FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/migrations ./backend/migrations
COPY backend/src ./backend/src
WORKDIR /app/backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/backend/recipe.json /app/backend/recipe.json
WORKDIR /app/backend
RUN cargo chef cook --release --recipe-path recipe.json

COPY backend/ /app/backend/
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    curl \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN mkdir -p /app/data /app/models /app/runtimes

COPY --from=backend-builder /app/backend/target/release/monolai /usr/local/bin/monolai
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/monolai

ENV HOST=0.0.0.0 \
    PORT=3000 \
    HARDWARE=auto \
    MODELS_DIR=/app/models \
    RUNTIMES_DIR=/app/runtimes \
    DATA_DIR=/app/data

EXPOSE 3000

VOLUME ["/app/models", "/app/runtimes", "/app/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://127.0.0.1:${PORT}/health || exit 1

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["monolai"]
