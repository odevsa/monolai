# --- Stage 1: Build Frontend ---
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# --- Stage 2: Cargo Chef Planner & Builder ---
FROM lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/migrations ./backend/migrations
COPY backend/src ./backend/src
WORKDIR /app/backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/backend/recipe.json /app/backend/recipe.json
WORKDIR /app/backend
RUN cargo chef cook --release --recipe-path recipe.json

# Copy backend source & built frontend assets into the expected path for rust-embed
COPY backend/ /app/backend/
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
RUN cargo build --release

# --- Stage 3: Minimal Runtime Image ---
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# Create persistent storage directories
RUN mkdir -p /app/data

COPY --from=backend-builder /app/backend/target/release/monolai /usr/local/bin/monolai
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/monolai

ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATA_DIR=/app/data

EXPOSE 3000

VOLUME ["/app/data"]

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["monolai"]
